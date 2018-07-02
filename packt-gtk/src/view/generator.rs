use gtk::{self, prelude::*};
use packt_core::{
    geometry::Rectangle,
    problem::{Generator, Problem, Variant},
};
use relm::{Relm, Update, Widget};

#[derive(Default)]
pub struct Model {
    problem: Option<Problem>,
}

#[derive(Msg)]
pub enum Msg {
    Toggle(Settings),
    Generate,
    Move,
    Moved(Problem),
}

#[derive(Clone, Copy)]
pub enum Settings {
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

struct Widgets {
    vbox: gtk::Box,
    settings: SettingsPanel,
    textview: gtk::TextView,
    move_btn: gtk::Button,
}

pub struct GeneratorWidget {
    relm: Relm<GeneratorWidget>,
    model: Model,
    widgets: Widgets,
}

impl Update for GeneratorWidget {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_relm: &Relm<Self>, _param: ()) -> Self::Model {
        Model::default()
    }

    fn update(&mut self, event: Self::Msg) {
        use self::Msg::*;
        match event {
            Toggle(c) => self.widgets.settings.toggle(c),
            Generate => self.generate_problem(),
            Move => self.relm.stream().emit(Msg::Moved(
                self.model.problem.take().expect("missing problem value"),
            )),
            Moved(_) => {
                self.widgets
                    .textview
                    .get_buffer()
                    .expect("failed to get buffer")
                    .set_text("");
                self.widgets.move_btn.set_sensitive(false);
            }
        }
    }
}

impl Widget for GeneratorWidget {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.widgets.vbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let builder = gtk::Builder::new_from_string(super::GLADE_SRC);
        let vbox = builder
            .get_object("generator_box")
            .expect("failed to get main_paned");
        let settings = SettingsPanel::from_builder(relm, &builder);

        let generate_btn: gtk::Button = builder
            .get_object("generate_button")
            .expect("failed to get generate_button");
        connect!(relm, generate_btn, connect_clicked(_), Msg::Generate);

        let move_btn: gtk::Button = builder
            .get_object("add_button")
            .expect("failed to get add_button");
        connect!(relm, move_btn, connect_clicked(_), Msg::Move);

        let textview: gtk::TextView = builder
            .get_object("problem_textview")
            .expect("failed to get problem_textview");

        GeneratorWidget {
            relm: relm.clone(),
            model,
            widgets: Widgets {
                vbox,
                settings,
                textview,
                move_btn,
            },
        }
    }
}

impl GeneratorWidget {
    fn generate_problem(&mut self) {
        self.widgets.move_btn.set_sensitive(true);

        let settings = &self.widgets.settings;
        let mut generator = Generator::new();
        if !settings.container_switch.get_active() {
            let width = settings.container_width_spinbtn.get_value_as_int() as u32;
            let height = settings.container_height_spinbtn.get_value_as_int() as u32;
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
        let text = problem.to_string();
        self.widgets
            .textview
            .get_buffer()
            .expect("failed to get buffer")
            .set_text(&text);
        self.model.problem = Some(problem);
    }
}

impl SettingsPanel {
    fn from_builder(relm: &Relm<GeneratorWidget>, builder: &gtk::Builder) -> Self {
        use self::Settings::*;
        let container_switch: gtk::CheckButton = builder.get_object("container_btn").unwrap();
        let container_filters_box = builder.get_object("container_filter_box").unwrap();
        let container_width_spinbtn = builder.get_object("container_width_spinbtn").unwrap();
        let container_height_spinbtn = builder.get_object("container_height_spinbtn").unwrap();
        connect!(
            relm,
            container_switch,
            connect_toggled(_),
            Msg::Toggle(Container)
        );

        let amount_switch: gtk::CheckButton = builder.get_object("amount_btn").unwrap();
        let amount_spinbtn = builder.get_object("amount_spinbtn").unwrap();
        connect!(relm, amount_switch, connect_toggled(_), Msg::Toggle(Amount));

        let variant_switch: gtk::CheckButton = builder.get_object("variant_btn").unwrap();
        let variant_btn_box = builder.get_object("variant_btn_box").unwrap();
        let variant_fixed_radio = builder.get_object("variant_fixed_rbtn").unwrap();
        let _free_radio: gtk::RadioButton = builder.get_object("variant_free_rbtn").unwrap();
        connect!(
            relm,
            variant_switch,
            connect_toggled(_),
            Msg::Toggle(Variant)
        );

        let rotation_switch: gtk::CheckButton = builder.get_object("rotation_btn").unwrap();
        let rotation_checkbtn = builder.get_object("rotation_checkbtn").unwrap();
        connect!(
            relm,
            rotation_switch,
            connect_toggled(_),
            Msg::Toggle(Rotation)
        );

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

    fn toggle(&mut self, s: Settings) {
        use self::Settings::*;
        match s {
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
