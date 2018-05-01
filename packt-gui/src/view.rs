use gtk::Window;
use gtk::{self, prelude::*};
use packt_core::domain::{self, problem};
use relm::{Relm, Update, Widget};
use std;

pub struct Model {
    generator: problem::Generator,
    problem: Option<domain::Problem>,
}

#[derive(Msg)]
pub enum Msg {
    Generate,
    Save,
    Quit,
}

pub struct Win {
    model: Model,
    window: gtk::Window,

    //    container_sw: gtk::Switch,
    //    container_attr_box: gtk::Box,
    //    container_width_sp: gtk::SpinButton,
    //    container_height_sp: gtk::SpinButton,
    //    nboxes_sw: gtk::Switch,
    //    nboxes_sp: gtk::SpinButton,
    //    variant_sw: gtk::Switch,
    //    variant_btn_box: gtk::ButtonBox,
    //    variant_fixed_radio: gtk::RadioButton,
    //    variant_free_radio: gtk::RadioButton,
    //    rotation_sw: gtk::Switch,
    //    rotation_check: gtk::CheckButton,
    problem_tv: gtk::TextView,
    generate_btn: gtk::Button,
    save_btn: gtk::Button,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_relm: &Relm<Self>, _param: ()) -> Self::Model {
        Model {
            problem: None,
            generator: problem::Generator::new(),
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::Generate => {
                self.save_btn.set_sensitive(true);
                let problem = self.model.generator.generate();
                let problem_text = problem.digest();

                self.model.problem = Some(problem);
                self.problem_tv
                    .get_buffer()
                    .expect("couldn't get buffer")
                    .set_text(&problem_text);
            }
            Msg::Save => {
                let dialog = gtk::FileChooserDialog::new(
                    Some("Save File"),
                    Some(&self.window),
                    gtk::FileChooserAction::Save,
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
                            .problem
                            .as_ref()
                            .unwrap()
                            .save(path)
                            .unwrap();
                    }
                }
                dialog.close();
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

        let window: gtk::Window = builder
            .get_object("main_window")
            .expect("couldn't get main_window");
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        let generate_btn: gtk::Button = builder
            .get_object("generate_button")
            .expect("couldn't get generate_button");
        connect!(
            relm,
            generate_btn,
            connect_clicked(_),
            Msg::Generate
        );

        let save_btn: gtk::Button = builder
            .get_object("save_button")
            .expect("couldn't get save_button");
        connect!(relm, save_btn, connect_clicked(_), Msg::Save);

        let problem_tv: gtk::TextView = builder
            .get_object("problem_textview")
            .expect("couldn't get problem_textview");

        window.show_all();
        Win {
            model,
            window,
            problem_tv,
            generate_btn,
            save_btn,
        }
    }
}
