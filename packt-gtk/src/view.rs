use crossbeam_channel::{self, Sender};
use failure::Error;
use gtk::{self, prelude::*};
use gtk::{ButtonsType, DialogFlags, FileChooserAction, MessageType};
use packt_core::domain::problem::{Generator, Variant};
use packt_core::domain::solution::Evaluation;
use packt_core::domain::Problem;
use packt_core::domain::{self, Rectangle, Solution};
use relm::{Relm, Update, Widget};
use std::process::{Command, Stdio};
use std::string::ToString;
use std::time::Duration;
use std::time::Instant;
use std::{self, thread};
use tokio::prelude::*;
use tokio_core::reactor::Core;
use tokio_io::io;
use tokio_process::{Child, CommandExt};

type EvalResult = Result<Evaluation, Error>;

#[derive(Default)]
pub struct Model {
    generated_problem: Option<domain::Problem>,
    selected_problem: Option<domain::Problem>,
}

#[derive(Msg)]
pub enum Msg {
    Toggle(Setting),
    Generate,
    Add,
    Import,
    Save,
    Completed(EvalResult),
    Run,
    Quit,
}

struct Widgets {
    window: gtk::Window,
    settings: SettingsPanel,
    problem_tv: gtk::TextView,
    runner_tv: gtk::TextView,
    add_btn: gtk::Button,
    save_btn: gtk::ToolButton,
    run_btn: gtk::Button,
    solver_filechooser: gtk::FileChooser,
}

pub struct Win {
    //    relm: Relm<Win>,
    model: Model,
    widgets: Widgets,
    sender: Sender<(domain::Problem, Command)>,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_relm: &Relm<Self>, _param: ()) -> Self::Model {
        Model { ..Model::default() }
    }

    fn update(&mut self, event: Self::Msg) {
        use self::Msg::*;
        match event {
            Toggle(c) => self.widgets.settings.toggle(c),
            Generate => self.generate_problem(),
            Add => self.add_problem(),
            Import => self.import_problem(),
            Completed(s) => self.display_evaluation(s),
            Save => self.save_problem(),
            Run => self.run_problem(),
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        use self::Setting::*;

        let glade_src = include_str!("packt.glade");
        let builder = gtk::Builder::new_from_string(glade_src);

        let window: gtk::Window = builder
            .get_object("main_window")
            .expect("failed to get main_window");
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        let settings = SettingsPanel::from_builder(&builder);
        connect!(
            relm,
            settings.container_switch,
            connect_toggled(_),
            Msg::Toggle(Container)
        );

        connect!(
            relm,
            settings.amount_switch,
            connect_toggled(_),
            Msg::Toggle(Amount)
        );

        connect!(
            relm,
            settings.variant_switch,
            connect_toggled(_),
            Msg::Toggle(Variant)
        );

        connect!(
            relm,
            settings.rotation_switch,
            connect_toggled(_),
            Msg::Toggle(Rotation)
        );

        let generate_btn: gtk::Button = builder
            .get_object("generate_button")
            .expect("failed to get generate_button");
        connect!(relm, generate_btn, connect_clicked(_), Msg::Generate);

        let add_btn: gtk::Button = builder
            .get_object("add_button")
            .expect("failed to get add_button");
        connect!(relm, add_btn, connect_clicked(_), Msg::Add);

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

        let problem_tv: gtk::TextView = builder
            .get_object("problem_textview")
            .expect("failed to get problem_textview");

        let runner_tv: gtk::TextView = builder
            .get_object("runner_textview")
            .expect("failed to get runner_textview");

        let solver_filechooser: gtk::FileChooser = builder
            .get_object("solver_filechooser")
            .expect("failed to get solver_filechooser");

        window.show_all();
        let tx = Win::launch_runner(relm);

        Win {
            //            relm: relm.clone(),
            sender: tx,
            model,
            widgets: Widgets {
                window,
                settings,
                problem_tv,
                runner_tv,
                add_btn,
                save_btn,
                run_btn,
                solver_filechooser,
            },
        }
    }
}

impl Win {
    fn error_dialog(&self, msg: &str) -> gtk::MessageDialog {
        gtk::MessageDialog::new(
            Some(&self.widgets.window),
            DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Warning,
            ButtonsType::Close,
            msg,
        )
    }

    fn info_dialog(&self, msg: &str) -> gtk::MessageDialog {
        gtk::MessageDialog::new(
            Some(&self.widgets.window),
            DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Info,
            ButtonsType::Close,
            msg,
        )
    }

    fn display_evaluation(&self, result: EvalResult) {
        let dialog = match result {
            Ok(evaluation) => {
                let msg = evaluation.to_string();
                self.info_dialog(&msg)
            }
            Err(e) => {
                self.error_dialog(&format!("Something went wrong: {:?}", e))
            }
        };

        dialog.run();
        dialog.close();
    }

    fn run_problem(&mut self) {
        let solver = match self.widgets.solver_filechooser.get_filename() {
            Some(solver) => solver,
            None => {
                let dialog = self.error_dialog("Please select a solver first");
                dialog.run();
                dialog.close();
                return;
            }
        };

        let mut child = Command::new("java");
        child
            .arg("-jar")
            .arg(solver)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let _ = self.sender.send((
            self.model.selected_problem.as_ref().unwrap().clone(),
            child,
        ));
    }

    fn save_problem(&mut self) {
        let dialog = gtk::FileChooserDialog::new(
            Some("Save File"),
            Some(&self.widgets.window),
            FileChooserAction::Save,
        );

        let cancel: i32 = gtk::ResponseType::Cancel.into();
        let accept: i32 = gtk::ResponseType::Accept.into();
        dialog.add_button("Cancel", cancel);
        dialog.add_button("Save", accept);

        if let Ok(p) = std::env::current_dir() {
            dialog.set_current_folder(p);
        } else if let Some(p) = std::env::home_dir() {
            dialog.set_current_folder(p);
        }

        if accept == dialog.run() {
            if let Some(path) = dialog.get_filename() {
                self.model
                    .selected_problem
                    .as_ref()
                    .unwrap()
                    .save(path)
                    .unwrap();
            }
        }
        dialog.close();
    }

    fn import_problem(&mut self) {
        let dialog = gtk::FileChooserDialog::new(
            Some("Import File"),
            Some(&self.widgets.window),
            FileChooserAction::Open,
        );

        let cancel: i32 = gtk::ResponseType::Cancel.into();
        let accept: i32 = gtk::ResponseType::Accept.into();
        dialog.add_button("Cancel", cancel);
        dialog.add_button("Open", accept);

        if let Ok(p) = std::env::current_dir() {
            dialog.set_current_folder(p);
        } else if let Some(p) = std::env::home_dir() {
            dialog.set_current_folder(p);
        }

        if accept == dialog.run() {
            if let Some(path) = dialog.get_filename() {
                self.model.selected_problem = Problem::from_path(path).ok();
                self.refresh_buffer();
            }
        }
        dialog.close();
    }

    fn refresh_buffer(&mut self) {
        fn refresh(tv: &gtk::TextView, problem: Option<&Problem>) {
            let text = problem.map(Problem::to_string).unwrap_or(String::new());
            tv.get_buffer()
                .expect("failed to get buffer")
                .set_text(&text);
        }

        refresh(
            &self.widgets.problem_tv,
            self.model.generated_problem.as_ref(),
        );
        refresh(
            &self.widgets.runner_tv,
            self.model.selected_problem.as_ref(),
        );
    }

    fn generate_problem(&mut self) {
        self.widgets.add_btn.set_sensitive(true);

        let settings = &self.widgets.settings;
        let mut generator = Generator::new();
        if !settings.container_switch.get_active() {
            let width =
                settings.container_width_spinbtn.get_value_as_int() as u32;
            let height =
                settings.container_height_spinbtn.get_value_as_int() as u32;
            generator.container(Rectangle::new(width, height));
        }

        if !settings.amount_switch.get_active() {
            let amount = settings.amount_spinbtn.get_value_as_int() as usize;
            generator.rectangles(amount);
        }

        if !settings.variant_switch.get_active() {
            let v = if settings.variant_fixed_radio.get_active() {
                Variant::Fixed(0)
            } else {
                Variant::Free
            };

            generator.variant(v);
        }

        if !settings.rotation_switch.get_active() {
            let r = settings.rotation_checkbtn.get_active();
            generator.allow_rotation(r);
        }

        let problem = generator.generate();
        self.model.generated_problem = Some(problem);
        self.refresh_buffer();
    }

    fn add_problem(&mut self) {
        self.widgets.run_btn.set_sensitive(true);
        self.widgets.save_btn.set_sensitive(true);
        self.model.selected_problem = self.model.generated_problem.clone();
        self.refresh_buffer();
    }

    fn launch_runner(relm: &Relm<Self>) -> Sender<(domain::Problem, Command)> {
        let stream = relm.stream().clone();
        let (tx, rx) = crossbeam_channel::unbounded();
        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            rx.iter().for_each(
                |(problem, mut command): (domain::Problem, Command)| {
                    let mut child = command
                        .spawn_async(&core.handle())
                        .expect("Failed to spawn child process");

                    let stdin =
                        child.stdin().take().expect("Failed to open stdin");

                    let child = io::write_all(
                        stdin,
                        problem.to_string().into_bytes(),
                    ).map(|_| child)
                        .and_then(Child::wait_with_output)
                        .from_err()
                        .and_then(|output| {
                            let output =
                                String::from_utf8_lossy(&output.stdout);
                            output.parse::<Solution>()
                        })
                        .map(|mut solution| {
                            solution.source(problem.source);
                            solution.evaluate()
                        })
                        .then(|result| -> Result<(), ()> {
                            stream.emit(Msg::Completed(result));
                            Ok(())
                        })
                        .deadline(Instant::now() + Duration::from_secs(300));

                    let _ = core.run(child);
                },
            )
        });
        tx
    }
}

pub enum Setting {
    Container,
    Amount,
    Variant,
    Rotation,
}

struct SettingsPanel {
    container_switch: gtk::CheckButton,
    container_filters_box: gtk::Box,
    container_width_spinbtn: gtk::SpinButton,
    container_height_spinbtn: gtk::SpinButton,
    amount_switch: gtk::CheckButton,
    amount_spinbtn: gtk::SpinButton,
    variant_switch: gtk::CheckButton,
    variant_btn_box: gtk::ButtonBox,
    variant_fixed_radio: gtk::RadioButton,
    rotation_switch: gtk::CheckButton,
    rotation_checkbtn: gtk::CheckButton,
}

impl SettingsPanel {
    fn from_builder(builder: &gtk::Builder) -> Self {
        let container_switch = builder.get_object("container_btn").unwrap();
        let container_filters_box =
            builder.get_object("container_filter_box").unwrap();
        let container_width_spinbtn =
            builder.get_object("container_width_spinbtn").unwrap();
        let container_height_spinbtn =
            builder.get_object("container_height_spinbtn").unwrap();
        let amount_switch = builder.get_object("amount_btn").unwrap();
        let amount_spinbtn = builder.get_object("amount_spinbtn").unwrap();
        let variant_switch = builder.get_object("variant_btn").unwrap();
        let variant_btn_box = builder.get_object("variant_btn_box").unwrap();
        let variant_fixed_radio =
            builder.get_object("variant_fixed_rbtn").unwrap();
        let _free_radio: gtk::RadioButton =
            builder.get_object("variant_free_rbtn").unwrap();
        let rotation_switch = builder.get_object("rotation_btn").unwrap();
        let rotation_checkbtn =
            builder.get_object("rotation_checkbtn").unwrap();

        SettingsPanel {
            container_switch,
            container_filters_box,
            container_width_spinbtn,
            container_height_spinbtn,
            amount_switch,
            amount_spinbtn,
            variant_switch,
            variant_btn_box,
            variant_fixed_radio,
            rotation_switch,
            rotation_checkbtn,
        }
    }

    fn toggle(&mut self, o: Setting) {
        use self::Setting::*;
        match o {
            Container => self
                .container_filters_box
                .set_sensitive(!self.container_switch.get_active()),
            Amount => self
                .amount_spinbtn
                .set_sensitive(!self.amount_switch.get_active()),
            Variant => self
                .variant_btn_box
                .set_sensitive(!self.variant_switch.get_active()),
            Rotation => self
                .rotation_checkbtn
                .set_sensitive(!self.rotation_switch.get_active()),
        }
    }
}
