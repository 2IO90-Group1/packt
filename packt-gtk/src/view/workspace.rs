use crossbeam_channel::{self, Sender};
use failure::Error;
use gtk::{self, prelude::*, Label};
use packt_core::domain::{solution::Evaluation, Problem, Solution};
use relm::{Relm, Update, Widget};
use std::{
    collections::VecDeque, fmt::{self, Formatter}, process::{Command, Stdio},
    result, string::ToString, thread, time::{Duration, Instant},
};
use tokio::prelude::*;
use tokio_core::reactor::Core;
use tokio_io;
use tokio_process::{Child, CommandExt};

type Job = (Entry, Command);
type Result<T> = result::Result<T, Error>;
type EvalResult = Result<Evaluation>;

#[derive(Debug)]
pub struct Entry {
    id: u16,
    name: String,
    problem: Problem,
    solutions: Vec<EvalResult>,
}

impl Entry {
    fn new(id: u16, problem: Problem) -> Self {
        let name = format!(
            "n={n} h={v} r={r}",
            v = problem.variant,
            r = if problem.allow_rotation { "yes" } else { "no" },
            n = problem.rectangles.len()
        );

        Entry {
            id,
            name,
            problem,
            solutions: Vec::new(),
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.id.eq(&other.id)
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut s = String::new();
        for solution in &self.solutions {
            let eval_string = match solution {
                Ok(eval) => eval.to_string(),
                Err(e) => format!("Error: {}", e),
            };

            s.push_str(&eval_string);
            s.push_str("\n\n");
        }

        s.push_str(&self.problem.digest());
        write!(f, "{}", s)
    }
}

struct Widgets {
    vbox: gtk::Box,
    problems_lb: gtk::ListBox,
    textview: gtk::TextView,
    remove_btn: gtk::ToolButton,
    save_btn: gtk::ToolButton,
    run_btn: gtk::Button,
    solver_chooser: gtk::FileChooser,
}

pub struct Model {
    id_gen: u16,
    problems: VecDeque<Entry>,
    work_queue: Sender<Job>,
    running: u32,
}

#[derive(Msg)]
pub enum Msg<E: fmt::Display> {
    Import,
    Add(Problem),
    Remove,
    Select,
    Save,
    Saved(Problem),
    Run,
    Completed(Entry),
    Err(E),
}

pub struct WorkspaceWidget {
    relm: Relm<WorkspaceWidget>,
    model: Model,
    widgets: Widgets,
}

impl Update for WorkspaceWidget {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg<Error>;

    fn model(relm: &Relm<Self>, _param: ()) -> Self::Model {
        Model {
            id_gen: 0,
            problems: VecDeque::new(),
            work_queue: launch_runner(relm),
            running: 0,
        }
    }

    fn update(&mut self, event: Self::Msg) {
        use self::Msg::*;

        let result = match event {
            // taken care of by root widget
            Import | Saved(_) => Ok(()),
            Run => self.run_problems(),
            Completed(entry) => self.problem_completed(entry),
            Select => {
                self.widgets.save_btn.set_sensitive(true);
                self.widgets.remove_btn.set_sensitive(true);
                Ok(())
            }
            Save => self
                .save_problem()
                .ok_or_else(|| format_err!("failed to save problem")),
            Add(problem) => {
                let id = self.model.id_gen;
                self.model.id_gen += 1;
                let entry = Entry::new(id, problem);
                self.widgets
                    .problems_lb
                    .insert(&Label::new(entry.name.as_str()), -1);
                self.widgets.problems_lb.show_all();
                self.model.problems.push_back(entry);
                self.widgets.run_btn.set_sensitive(true);
                Ok(())
            }
            Remove => {
                if let Some(row) = self.widgets.problems_lb.get_selected_row() {
                    let i = row.get_index();
                    self.widgets.problems_lb.remove(&row);
                    self.model.problems.remove(i as usize);
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Something went wrong: {}", e);
                Ok(())
            }
        };

        if let result::Result::Err(e) = result {
            self.relm.stream().emit(Err(e))
        }

        let _ = self.refresh_buffer();
        if self.widgets.problems_lb.get_selected_row() == None {
            self.widgets.remove_btn.set_sensitive(false);
            self.widgets.save_btn.set_sensitive(false);
        }
    }
}

impl Widget for WorkspaceWidget {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.widgets.vbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let builder = gtk::Builder::new_from_string(&super::GLADE_SRC);
        let vbox = builder
            .get_object("workspace_box")
            .expect("failed to get workspace_box");
        let problems_lb: gtk::ListBox = builder
            .get_object("workspace_listbox")
            .expect("failed to get workspace_listbox");

        connect!(relm, problems_lb, connect_row_selected(_, _), {
            Msg::Select
        });

        let remove_btn: gtk::ToolButton = builder
            .get_object("remove_problem_btn")
            .expect("failed to get remove_problem_btn");
        connect!(relm, remove_btn, connect_clicked(_), Msg::Remove);

        let textview = builder
            .get_object("runner_textview")
            .expect("failed to get runner_textview");

        let save_btn: gtk::ToolButton = builder
            .get_object("save_problem_btn")
            .expect("failed to get save_problem_btn");
        connect!(relm, save_btn, connect_clicked(_), Msg::Save);

        let import_btn: gtk::ToolButton = builder
            .get_object("import_problem_btn")
            .expect("failed to get import_problem_btn");
        connect!(relm, import_btn, connect_clicked(_), Msg::Import);

        let run_btn: gtk::Button = builder
            .get_object("run_button")
            .expect("failed to get run_button");
        connect!(relm, run_btn, connect_clicked(_), Msg::Run);

        let solver_chooser: gtk::FileChooser = builder
            .get_object("solver_filechooser")
            .expect("failed to get solver_filechooser");

        WorkspaceWidget {
            relm: relm.clone(),
            model,
            widgets: Widgets {
                vbox,
                problems_lb,
                textview,
                remove_btn,
                save_btn,
                run_btn,
                solver_chooser,
            },
        }
    }
}

impl WorkspaceWidget {
    fn save_problem(&mut self) -> Option<()> {
        let entry = self
            .widgets
            .problems_lb
            .get_selected_row()
            .map(|row| {
                let index = row.get_index() as usize;
                self.model.problems.get(index).unwrap().clone()
            })
            .or_else(|| {
                self.widgets.problems_lb.get_selected_row().map(|row| {
                    let index = row.get_index() as usize;
                    self.model.problems.get(index).unwrap().clone()
                })
            })?;

        self.relm.stream().emit(Msg::Saved(entry.problem.clone()));
        Some(())
    }

    fn run_problems(&mut self) -> Result<()> {
        if self.model.running != 0 {
            bail!("failed to start new jobs -- there are still jobs running");
        }

        let solver = match self.widgets.solver_chooser.get_filename() {
            Some(solver) => solver,
            None => {
                bail!("Please select a solver first");
            }
        };

        self.model.running += self.model.problems.len() as u32;
        for p in self.model.problems.drain(..) {
            let mut command = Command::new("java");
            command
                .arg("-jar")
                .arg(&solver)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());

            if let Err(_) = self.model.work_queue.send((p, command)) {
                bail!("failed to enqueue job");
            }
        }

        Ok(())
    }

    fn problem_completed(&mut self, entry: Entry) -> Result<()> {
        self.model.problems.push_back(entry);
        self.model.running -= 1;
        self.refresh_buffer()?;

        println!("success");
        if self.model.running == 0 {
            println!("All jobs finished");
        }

        Ok(())
    }

    fn refresh_buffer(&mut self) -> Result<()> {
        let text =
            if let Some(row) = self.widgets.problems_lb.get_selected_row() {
                let i = row.get_index() as usize;
                println!("i: {}, problems: {:?}", i, self.model.problems.iter().map(|e| e.id).collect::<Vec<_>>());
                if let Some(p) = self.model.problems.get(i) {
                        p.to_string()
                    } else {
                        String::new()
                    }
            } else {
                String::new()
            };

        self.widgets
            .textview
            .get_buffer()
            .ok_or_else(|| format_err!("failed to get buffer"))?
            .set_text(text.as_ref());

        Ok(())
    }
}

fn launch_runner(relm: &Relm<WorkspaceWidget>) -> Sender<Job> {
    let stream = relm.stream().clone();
    let (tx, rx) = crossbeam_channel::unbounded();
    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        rx.iter().for_each(|(mut entry, mut command): Job| {
            let mut child = command
                .spawn_async(&core.handle())
                .expect("Failed to spawn child process");

            let stdin = child.stdin().take().expect("Failed to open stdin");
            let input = entry.problem.to_string();
            let source = entry.problem.source;

            let start = Instant::now();
            let child = tokio_io::io::write_all(stdin, input)
                .map(|_| child)
                .and_then(Child::wait_with_output)
                .from_err()
                .and_then(|output| {
                    let output = String::from_utf8_lossy(&output.stdout);
                    output.parse::<Solution>()
                })
                .map(|mut solution| {
                    solution.source(source);
                    solution.evaluate(start)
                })
                .then(|result| -> result::Result<(), ()> {
                    entry.solutions.push(result);
                    stream.emit(Msg::Completed(entry));
                    Ok(())
                })
                .deadline(start + Duration::from_secs(300));

            let _ = core.run(child);
        })
    });
    tx
}
