mod style;
mod time;
use grid::Grid;

use iced::{
    button, executor, Application, Button, Clipboard, Column, Command, Container, Element, Length,
    Row, Settings, Subscription, Text,
};
use std::time::Duration;
use time::Timer;

const FPS: u64 = 30;
const MILLISEC: u64 = 1000;

fn main() {
    GameOfLife::run(Settings::default()).unwrap();
}

#[derive(Default)]
struct GameOfLife {
    grid: Grid,
    is_playing: bool,
    toggle_button: button::State,
    next_button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    Grid(grid::Message),
    Tick,
    Toggle,
    Next,
}

impl Application for GameOfLife {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                grid: Grid::new(),
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Game of Life")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::Grid(message) => (),
            Message::Tick | Message::Next => {
                self.grid.tick();
            }
            Message::Toggle => {
                self.is_playing = !self.is_playing;
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let control = Row::new()
            .spacing(10)
            .push(
                Button::new(
                    &mut self.toggle_button,
                    Text::new(if self.is_playing { "Pause" } else { "Play" }),
                )
                .on_press(Message::Toggle)
                .style(style::Button),
            )
            .push(
                Button::new(&mut self.next_button, Text::new("Next"))
                    .on_press(Message::Next)
                    .style(style::Button),
            );

        let content = Column::new()
            .spacing(10)
            .padding(10)
            .push(self.grid.view().map(Message::Grid))
            .push(control);
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Container)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.is_playing {
            let timer = Timer::new(Duration::from_millis(MILLISEC / FPS));
            Subscription::from_recipe(timer).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

mod grid {
    use iced::canvas::{self, Canvas, Cursor, Geometry, Path};
    use iced::{Color, Element, Length, Point, Rectangle, Size, Vector};
    use rand::Rng;

    const SIZE: usize = 32;

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum Cell {
        Unpopulated,
        Populated,
    }

    impl Default for Cell {
        fn default() -> Self {
            Cell::Unpopulated
        }
    }

    #[derive(Debug, Clone)]
    pub enum Message {}

    #[derive(Default)]
    pub struct Grid {
        cells: [[Cell; SIZE]; SIZE],
        cache: canvas::Cache,
    }

    impl Grid {
        pub fn new() -> Self {
            let mut rng = rand::thread_rng();
            let mut cells = [[Cell::default(); SIZE]; SIZE];
            for x in 0..SIZE {
                for y in 0..SIZE {
                    if rng.gen_range(0..3) == 0 {
                        cells[x][y] = Cell::Populated;
                    }
                }
            }
            Self {
                cells,
                ..Default::default()
            }
        }

        pub fn tick(&mut self) {
            let mut populated_neighbors: [[usize; SIZE]; SIZE] = [[0; SIZE]; SIZE];

            for (i, row) in self.cells.iter().enumerate() {
                for (j, _) in row.iter().enumerate() {
                    populated_neighbors[i][j] = self.populated_neighbors(i, j);
                }
            }

            for (i, row) in populated_neighbors.iter().enumerate() {
                for (j, amount) in row.iter().enumerate() {
                    let is_populated = self.cells[i][j] == Cell::Populated;

                    self.cells[i][j] = match amount {
                        2 if is_populated => Cell::Populated,
                        3 => Cell::Populated,
                        _ => Cell::Unpopulated,
                    };
                }
            }

            self.cache.clear();
        }

        pub fn view(&mut self) -> Element<Message> {
            Canvas::new(self)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }

        fn populated_neighbors(&self, row: usize, column: usize) -> usize {
            use itertools::Itertools;

            let rows = row.saturating_sub(1)..=row + 1;
            let columns = column.saturating_sub(1)..=column + 1;

            let is_inside_bounds = |i: usize, j: usize| i < SIZE && j < SIZE;
            let is_neighbor = |i: usize, j: usize| i != row || j != column;

            let is_populated = |i: usize, j: usize| self.cells[i][j] == Cell::Populated;

            rows.cartesian_product(columns)
                .filter(|&(i, j)| is_inside_bounds(i, j) && is_neighbor(i, j) && is_populated(i, j))
                .count()
        }

        fn region(&self, size: Size) -> Rectangle {
            let side = size.width.min(size.height);

            Rectangle {
                x: (size.width - side) / 2.0,
                y: (size.height - side) / 2.0,
                width: side,
                height: side,
            }
        }
    }

    impl canvas::Program<Message> for Grid {
        fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
            let region = self.region(bounds.size());
            let cell_size = Size::new(1.0, 1.0);

            let life = self.cache.draw(bounds.size(), |frame| {
                let background = Path::rectangle(region.position(), region.size());
                frame.fill(
                    &background,
                    Color::from_rgb(
                        0x40 as f32 / 255.0,
                        0x44 as f32 / 255.0,
                        0x4B as f32 / 255.0,
                    ),
                );

                frame.with_save(|frame| {
                    frame.translate(Vector::new(region.x, region.y));
                    frame.scale(region.width / SIZE as f32);

                    let cells = Path::new(|p| {
                        for (i, row) in self.cells.iter().enumerate() {
                            for (j, cell) in row.iter().enumerate() {
                                if *cell == Cell::Populated {
                                    p.rectangle(Point::new(j as f32, i as f32), cell_size);
                                }
                            }
                        }
                    });
                    frame.fill(&cells, Color::WHITE);
                });
            });

            vec![life]
        }
    }
}
