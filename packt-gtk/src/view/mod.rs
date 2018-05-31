mod generator;
mod workspace;

use self::generator::GeneratorWidget;
use self::workspace::WorkspaceWidget;

use gtk::{
    self, prelude::*, ButtonsType, DialogFlags, FileChooserAction, MessageType,
};
use packt_core::domain::Problem;
use relm::{Component, ContainerWidget, Relm, Update, Widget};
use std::{self, fmt, path::PathBuf};

const GLADE_SRC: &str = include_str!("../packt.glade");

#[derive(Msg)]
pub enum Msg<E: fmt::Display> {
    Import,
    Save(Problem),
    Err(E),
    Quit,
}

struct Widgets {
    _generator: Component<GeneratorWidget>,
    workspace: Component<WorkspaceWidget>,
    window: gtk::Window,
}

pub struct Win {
    widgets: Widgets,
    // relm: Relm<Win>,
}

impl Update for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg<String>;

    fn model(_relm: &Relm<Self>, _param: ()) -> Self::Model {
        ()
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Save(problem) => self.save_problem(&problem),
            Msg::Import => self.import_problem(),
            Msg::Quit => gtk::main_quit(),
            Msg::Err(e) => {
                let dialog = self.error_dialog(e);
                dialog.run();
                dialog.close();
            }
        }
    }
}

impl Widget for Win {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
        use self::generator::Msg::*;
        use self::workspace::Msg::*;

        let builder = gtk::Builder::new_from_string(&GLADE_SRC);
        let window: gtk::Window = builder
            .get_object("main_window")
            .expect("failed to get main_window");
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        let paned: gtk::Paned = builder
            .get_object("main_paned")
            .expect("failed to get main_paned");

        let _generator = paned.add_widget::<GeneratorWidget>(());
        let workspace = paned.add_widget::<WorkspaceWidget>(());
        connect!(_generator@Moved(ref problem), workspace, Add(problem.clone()));
        connect!(workspace@Import, relm, Msg::Import);
        connect!(workspace@Saved(ref problem), relm, Msg::Save(problem.clone()));
        connect!(workspace@Err(ref e), relm, Msg::Err(e.to_string()));

        window.show_all();
        Win {
            // relm: relm.clone(),
            widgets: Widgets {
                _generator,
                workspace,
                window,
            },
        }
    }
}

impl Win {
    fn error_dialog<M: AsRef<str>>(&self, msg: M) -> gtk::MessageDialog {
        gtk::MessageDialog::new(
            Some(&self.widgets.window),
            DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Warning,
            ButtonsType::Close,
            msg.as_ref(),
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

    fn filechooser_dialog(&self, action: FileChooserAction) -> Option<PathBuf> {
        let (title, accept_text) = match action {
            FileChooserAction::Save => ("Save file", "Save"),
            FileChooserAction::Open => ("Open file", "Open"),
            _ => unreachable!(),
        };

        let dialog = gtk::FileChooserDialog::new(
            title.into(),
            Some(&self.widgets.window),
            FileChooserAction::Save,
        );

        let cancel: i32 = gtk::ResponseType::Cancel.into();
        let accept: i32 = gtk::ResponseType::Accept.into();
        dialog.add_button("Cancel", cancel);
        dialog.add_button(accept_text, accept);

        if let Ok(p) = std::env::current_dir() {
            dialog.set_current_folder(p);
        } else if let Some(p) = std::env::home_dir() {
            dialog.set_current_folder(p);
        }

        let result = if accept == dialog.run() {
            dialog.get_filename()
        } else {
            None
        };

        dialog.close();
        result
    }

    fn save_problem(&mut self, problem: &Problem) {
        if let Some(path) = self.filechooser_dialog(FileChooserAction::Save) {
            problem.save(path).unwrap();
        }
    }

    fn import_problem(&mut self) {
        if let Some(path) = self.filechooser_dialog(FileChooserAction::Open) {
            match Problem::from_path(path) {
                Ok(problem) => {
                    self.widgets.workspace.emit(workspace::Msg::Add(problem));
                }
                Err(_e) => (), /* self.relm.stream().emit(Msg::Err(e.
                                * to_string())), */
            }
        }
    }
}
