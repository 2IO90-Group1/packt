use crossbeam_channel::{self, Sender};
use failure::Error;
use gtk::{self, prelude::*, Label};
use packt_core::{
    problem::Problem, runner, solution::{Evaluation},
};

use relm::{Relm, Update, Widget};
use std::{
    collections::VecDeque, env, fmt::{self, Formatter}, path::PathBuf,
    result, string::ToString, sync::atomic::{AtomicU32, Ordering}, thread,
};
use tokio::prelude::*;
use tokio_core::reactor::Core;

type Job = (usize, PathBuf, String);
type Result<T> = result::Result<T, Error>;
type EvalResult = Result<Evaluation>;

#[derive(Debug)]
pub struct Entry {
    id: usize,
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
            id: 0,
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
    retry_spinbtn: gtk::SpinButton,
    threshold_spinbtn: gtk::SpinButton,
    nwidths_spinbtn: gtk::SpinButton,
}

pub struct Model {
    problems: VecDeque<Entry>,
    work_queue: Sender<Job>,
    running: AtomicU32,
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
    Completed(usize, EvalResult),
    Error(E),
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
            running: AtomicU32::new(0),
        }
    }

    fn update(&mut self, event: Self::Msg) {
        use self::Msg::*;

        let result = match event {
            // taken care of by root widget
            Import | Saved(_) => Ok(()),
            Run => self.run_problems(),
            Completed(id, result) => self.problem_completed(id, result),
            Select => {
                self.widgets.save_btn.set_sensitive(true);
                self.widgets.remove_btn.set_sensitive(true);
                Ok(())
            }
            Save => self
                .save_problem()
                .ok_or_else(|| format_err!("failed to save problem")),
            Add(_) | Remove => match (event, self.model.running.load(Ordering::SeqCst)) {
                (Add(problem), 0) => {
                    let entry = Entry::new(problem);
                    self.widgets
                        .problems_lb
                        .insert(&Label::new(entry.name.as_str()), -1);
                    self.widgets.problems_lb.show_all();
                    self.model.problems.push_back(entry.into());
                    self.widgets.run_btn.set_sensitive(true);
                    Ok(())
                }
                (Remove, 0) => {
                    if let Some(row) = self.widgets.problems_lb.get_selected_row() {
                        let i = row.get_index();
                        self.widgets.problems_lb.remove(&row);
                        self.model.problems.remove(i as usize);
                        Ok(())
                    } else {
                        Err(format_err!("Selected row does not exist"))
                    }
                }
                (_, x) => Err(format_err!(
                    "New problems cannot be added while the solver is running: {} problems running",
                    x
                )),
            },
            Error(e) => {
                eprintln!("Something went wrong: {}", e);
                Ok(())
            }
        };

        if let Err(e) = result {
            self.relm.stream().emit(Error(e))
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

        let retry_spinbtn = builder
            .get_object("retry_spinbtn")
            .expect("failed to get retry_spinbtn");

        let threshold_spinbtn = builder
            .get_object("threshold_spinbtn")
            .expect("failed to get threshold_spinbtn");

        let nwidths_spinbtn = builder
            .get_object("nwidths_spinbtn")
            .expect("failed to get nwidths_spinbtn");

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
                retry_spinbtn,
                threshold_spinbtn,
                nwidths_spinbtn,
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
            .and_then(|row| {
                let index = row.get_index() as usize;
                self.model.problems.get(index)
            })?;
        self.relm.stream().emit(Msg::Saved(entry.problem.clone()));
        Some(())
    }

    fn run_problems(&mut self) -> Result<()> {
        if self.model.running.load(Ordering::SeqCst) != 0 {
            bail!("failed to start new jobs -- there are still jobs running");
        }

        let solver = match self.widgets.solver_chooser.get_filename() {
            Some(solver) => solver,
            None => bail!("Please select a solver first"),
        };

        let retry = self.widgets.retry_spinbtn.get_value_as_int();
        let threshold = self.widgets.threshold_spinbtn.get_value();
        let nheights = self.widgets.nwidths_spinbtn.get_value_as_int();

        env::set_var("RETRY", retry.to_string());
        env::set_var("THRESHOLD", threshold.to_string());
        env::set_var("N_HEIGHTS", nheights.to_string());

        *self.model.running.get_mut() = self.model.problems.len() as u32;
        for (i, problem) in self
            .model
            .problems
            .iter()
            .map(|e| e.problem.to_string())
            .enumerate()
        {
            if let Err(_) = self.model.work_queue.send((i, solver.clone(), problem)) {
                bail!("failed to enqueue job");
            }
        }

        Ok(())
    }

    fn problem_completed(&mut self, id: usize, result: EvalResult) -> Result<()> {
        let old = self.model.running.fetch_sub(1, Ordering::SeqCst);
        self.model.problems[id].solutions.push(result);
        self.refresh_buffer()?;

        eprintln!("success");
        if old == 1 {
            eprintln!("All jobs finished");
        }

        Ok(())
    }

    fn refresh_buffer(&mut self) -> Result<()> {
        let text = if let Some(row) = self.widgets.problems_lb.get_selected_row() {
            let i = row.get_index() as usize;
            self.model.problems[i].to_string()
        } else {
            "not found".to_string()
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
        rx.iter().for_each(|(id, solver, problem)| {
            let handle = core.handle();
            let child = runner::solve_async(&solver, problem, handle).then(
                |result| -> result::Result<(), ()> {
                    stream.emit(Msg::Completed(id, result));
                    Ok(())
                },
            );

            let _ = core.run(child);
        })
    });
    tx
}
