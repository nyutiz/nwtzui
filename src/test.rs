use iced::Center;
use iced::widget::{Column, button, column, text};

pub fn main() -> iced::Result {
    iced::run("A cool counter", State::update, State::view)
}

#[derive(Default)]
struct State {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Increment,
    Decrement,
}

impl State {
    fn update(&mut self, message: Action) {
        match message {
            Action::Increment => {
                self.value += 1;
            }
            Action::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Column<Action> {
        column![
            button("Increment").on_press(Action::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Action::Decrement)
        ]
            .padding(20)
            .align_x(Center)
    }
}