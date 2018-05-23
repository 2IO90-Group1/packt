use crossbeam_channel::{self, Sender};
use failure::Error;
use gtk::{self, prelude::*, Label, ListBox};
use packt_core::domain::{solution::Evaluation, Problem, Solution};
use relm::{Relm, Update, Widget};
use std::{
    collections::VecDeque, fmt::{self, Formatter}, mem, process::{Command, Stdio}, result,
    string::ToString, thread, time::{Duration, Instant},
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
    name: String,
    problem: Problem,
    solutions: Vec<EvalResult>,
}

impl Entry {
    fn new(problem: Problem) -> Self {
        let name = format!(
            "n={n} h={v} r={r}",
            v = problem.variant,
            r = if problem.allow_rotation { "yes" } else { "no" },
            n = problem.rectangles.len()
        );

        Entry {
            name,
            problem,
            solutions: Vec::new(),
        }
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
    selected_lb: gtk::ListBox,
    textview: gtk::TextView,
    select_btn: gtk::ToolButton,
    unselect_btn: gtk::ToolButton,
    save_btn: gtk::ToolButton,
    import_btn: gtk::ToolButton,
    run_btn: gtk::Button,
    solver_chooser: gtk::FileChooser,
}

pub struct Model {
    problems: VecDeque<Entry>,
    selected: VecDeque<Entry>,
    work_queue: Sender<Job>,
    running: u32,
}

#[derive(Msg)]
pub enum Msg<E: fmt::Display> {
    Import,
    Add(Problem),
    Select(List),
    Enqueue,
    Dequeue,
    Save,
    Saved(Problem),
    Run,
    Completed(Entry),
    Err(E),
}

#[derive(PartialEq)]
pub enum List {
    Problems,
    Selected,
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
            Select(list) => self.clicked(list),
            Enqueue => self
                .move_selected(false)
                .ok_or_else(|| format_err!("failed to move problems")),
            Dequeue => self
                .move_selected(true)
                .ok_or_else(|| format_err!("failed to move problem")),
            Run => self.run_problem(),
            Save => self
                .save_problem()
                .ok_or_else(|| format_err!("failed to save problem")),
            Completed(entry) => self.problem_completed(entry),
            Add(problem) => {
                let entry = Entry::new(problem);
                self.widgets
                    .problems_lb
                    .prepend(&Label::new(entry.name.as_str()));
                self.widgets.problems_lb.show_all();
                self.model.problems.push_front(entry);
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

        let selected_lb: gtk::ListBox = builder
            .get_object("selection_listbox")
            .expect("failed to get selection_listbox");

        let select_btn: gtk::ToolButton = builder
            .get_object("select_problem_btn")
            .expect("failed to get select_problem_btn");
        connect!(relm, select_btn, connect_clicked(_), Msg::Enqueue);

        let unselect_btn: gtk::ToolButton = builder
            .get_object("unselect_problem_btn")
            .expect("failed to get unselect_problem_btn");
        connect!(relm, unselect_btn, connect_clicked(_), Msg::Dequeue);

        let problems_lb_clone = problems_lb.clone();
        let select_btn_clone = select_btn.clone();
        connect!(relm, problems_lb, connect_row_selected(_, row), {
            select_btn_clone.set_sensitive(row.is_some());
            Msg::Select(List::Problems)
        });

        let selected_lb_clone = selected_lb.clone();
        let unselect_btn_clone = unselect_btn.clone();
        connect!(relm, selected_lb, connect_row_selected(_, row), {
            unselect_btn_clone.set_sensitive(row.is_some());
            Msg::Select(List::Selected)
        });

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
                selected_lb,
                textview,
                select_btn,
                unselect_btn,
                save_btn,
                import_btn,
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
                self.widgets.selected_lb.get_selected_row().map(|row| {
                    let index = row.get_index() as usize;
                    self.model.selected.get(index).unwrap().clone()
                })
            })?;

        self.relm.stream().emit(Msg::Saved(entry.problem.clone()));
        Some(())
    }

    fn run_problem(&mut self) -> Result<()> {
        if self.model.running != 0 {
            bail!("failed to start new jobs -- there are still jobs running");
        }

        let solver = match self.widgets.solver_chooser.get_filename() {
            Some(solver) => solver,
            None => {
                bail!("Please select a solver first");
            }
        };

        for p in self.model.selected.drain(..) {
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

        self.model.running += self.model.selected.len() as u32;
        Ok(())
    }

    fn problem_completed(&mut self, entry: Entry) -> Result<()> {
        println!("success");
        let index = self.model.selected.len();
        let text = entry.name.clone();
        self.model.selected.push_back(entry);
        self.widgets
            .selected_lb
            .get_row_at_index(index as i32)
            .ok_or_else(|| format_err!("failed to get row in selected_lb"))?
            .get_child()
            .ok_or_else(|| format_err!("failed to get child of row"))?
            .downcast::<Label>()
            .map_err(|e| format_err!("failed to downcast to label: {:?}", e))?
            .set_text(&text);
        Ok(())
    }

    fn clicked(&mut self, lb: List) -> Result<()> {
        let lb = if lb == List::Problems { self.widgets.problems_lb.clone() } else { self.widgets.selected_lb.clone() };
        if let Some(row) = lb.get_selected_row() {
            let list = if lb == self.widgets.problems_lb {
                &mut self.model.problems
            } else {
                &mut self.model.selected
            };
            let i = row.get_index() as usize;
            println!("{:#?}", list);
            let entry =
                list.get(i).ok_or_else(|| format_err!("model invalid"))?;
            self.widgets
                .textview
                .get_buffer()
                .ok_or_else(|| format_err!("failed to get buffer"))?
                .set_text(entry.to_string().as_ref());
        }

        Ok(())
    }

    fn move_selected(&mut self, reversed: bool) -> Option<()> {
        let mut from_lb = self.widgets.problems_lb.clone();
        let from_vec = &mut self.model.problems;

        let mut to_lb = self.widgets.selected_lb.clone();
        let to_vec = &mut self.model.selected;

        if reversed {
            mem::swap(&mut from_lb, &mut to_lb);
            mem::swap(from_vec, to_vec);
        }

        let row = from_lb.get_selected_row()?;
        let i = row.get_index() as usize;
        to_vec.push_front(from_vec.remove(i)?);
        from_lb.remove(&row);
        to_lb.add(&row);
        self.widgets
            .run_btn
            .set_sensitive(!self.model.selected.is_empty());
        Some(())
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
