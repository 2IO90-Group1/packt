use gtk::{self, prelude::*};
use gtk::{Button, TextView, Window};
use packt_core::domain;
use relm::{Relm, Update, Widget};

pub struct Model {
    problem: Option<domain::Problem>,
}

#[derive(Msg)]
pub enum Msg {
    Generate,
    Quit,
}

pub struct Win {
    model: Model,
    window: Window,
    button: Button,
    textview: TextView,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_relm: &Relm<Self>, _param: ()) -> Self::Model {
        Model { problem: None }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Generate => {
                let problem = domain::Problem::generator().generate();
                let problem_text = problem.digest();

                self.model.problem = Some(problem);
                self.textview
                    .get_buffer()
                    .expect("couldn't get buffer")
                    .set_text(&problem_text);
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }
    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let glade_src = include_str!("packt.glade");
        let builder = gtk::Builder::new_from_string(glade_src);

        let window: Window = builder
            .get_object("main_window")
            .expect("couldn't get main_window");
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        let button: Button = builder
            .get_object("generate_button")
            .expect("couldn't get generate_button");
        connect!(relm, button, connect_clicked(_), Msg::Generate);

        let textview = builder
            .get_object("problem_textview")
            .expect("couldn't get problem_textview");

        window.show_all();
        Win {
            model,
            window,
            button,
            textview,
        }
    }
}
