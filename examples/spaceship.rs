use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

use tetra::graphics::scaling::{ScalingMode, ScreenScaler};

use tetra::graphics::text::{Font, Text};

use tetra::input::{self, Key};

use mun_runtime::{invoke_fn, RetryResultExt, RuntimeBuilder, StructRef};

use std::cell::RefCell;
use std::rc::Rc;

use rand::prelude::*;

extern "C" fn sin(number: f32) -> f32 {
    number.sin()
}

extern "C" fn cos(number: f32) -> f32 {
    number.cos()
}

extern "C" fn dbg(number: f32) {
    dbg!(number);
}

extern "C" fn degrees_to_radians(degrees: f32) -> f32 {
    degrees.to_radians()
}

extern "C" fn sqrt(value: f32) -> f32 {
    value.sqrt()
}

extern "C" fn game_area_width() -> f32 {
    128.0 * 5.0
}

extern "C" fn game_area_height() -> f32 {
    72.0 * 5.0
}

fn textures(ctx: &mut Context) -> [(Texture, Vec2<f32>); 5] {
    [
        (
            Texture::new(ctx, "./assets/spaceship/sprites/spaceship.png").unwrap(),
            Vec2::new(6., 7.),
        ),
        (
            Texture::new(ctx, "./assets/spaceship/sprites/rocket.png").unwrap(),
            Vec2::new(3., 3.),
        ),
        (
            Texture::new(ctx, "./assets/spaceship/sprites/asteroid_size_1.png").unwrap(),
            Vec2::new(5.0, 5.0),
        ),
        (
            Texture::new(ctx, "./assets/spaceship/sprites/asteroid_size_2.png").unwrap(),
            Vec2::new(8.0, 8.0),
        ),
        (
            Texture::new(ctx, "./assets/spaceship/sprites/asteroid_size_3.png").unwrap(),
            Vec2::new(15.0, 15.0),
        ),
    ]
}

struct SpaceshipGame {
    mun_runtime: Rc<RefCell<mun_runtime::Runtime>>,
    asteroids: Vec<StructRef>,
    rockets: Vec<StructRef>,
    textures: [(Texture, Vec2<f32>); 5],
    scaler: ScreenScaler,
    game_struct: StructRef,
    player_input: StructRef,
    font: Font,
    score: u8,
}

impl State for SpaceshipGame {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());

        graphics::clear(ctx, Color::BLACK);

        let spaceship_object: StructRef = self
            .game_struct
            .get::<StructRef>("spaceship")
            .unwrap()
            .get::<StructRef>("object")
            .unwrap();
        let spaceship_object_position = spaceship_object.get::<StructRef>("position").unwrap();

        // Draw rockets
        for rocket in self.rockets.iter() {
            let rocket_object = rocket.get::<StructRef>("object").unwrap();
            let rocket_position = rocket_object.get::<StructRef>("position").unwrap();

            graphics::draw(
                ctx,
                &self.textures[1].0,
                DrawParams::new()
                    .position(Vec2::new(
                        rocket_position.get("x").unwrap(),
                        rocket_position.get("y").unwrap(),
                    ))
                    .origin(self.textures[1].1)
                    .rotation(rocket_object.get::<f32>("angle").unwrap().to_radians()),
            );
        }

        // Draw spaceship
        graphics::draw(
            ctx,
            &self.textures[0].0,
            DrawParams::new()
                .position(Vec2::new(
                    spaceship_object_position.get("x").unwrap(),
                    spaceship_object_position.get("y").unwrap(),
                ))
                .origin(self.textures[0].1)
                .rotation(spaceship_object.get::<f32>("angle").unwrap().to_radians()),
        );

        // Draw asteroids
        for asteroid in self.asteroids.iter() {
            let asteroid_object = asteroid.get::<StructRef>("object").unwrap();
            let asteroid_position = asteroid_object.get::<StructRef>("position").unwrap();
            let asteroid_size: usize = asteroid.get::<u8>("size").unwrap().into();

            graphics::draw(
                ctx,
                &self.textures[asteroid_size + 1].0,
                DrawParams::new()
                    .position(Vec2::new(
                        asteroid_position.get("x").unwrap(),
                        asteroid_position.get("y").unwrap(),
                    ))
                    .origin(self.textures[asteroid_size + 1].1)
                    .rotation(asteroid_object.get::<f32>("angle").unwrap().to_radians()),
            );
        }

        graphics::reset_canvas(ctx);

        graphics::draw(ctx, &self.scaler, Vec2::new(0., 0.));

        // Draw score
        graphics::draw(
            ctx,
            &Text::new(format!("Score {}", self.score), self.font.clone()),
            Vec2::new(10., 10.),
        );

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        // Collect input to pass it into mun runtime
        if input::is_key_down(ctx, Key::Left) {
            self.player_input.set("left", true).unwrap();
        }
        if input::is_key_down(ctx, Key::Right) {
            self.player_input.set("right", true).unwrap();
        }
        if input::is_key_down(ctx, Key::Up) {
            self.player_input.set("up", true).unwrap();
        }
        if input::is_key_down(ctx, Key::Z) {
            self.player_input.set("shoot", true).unwrap();
        }

        if self.game_struct.get::<bool>("spawn_new_rocket").unwrap() {
            self.game_struct.set("spawn_new_rocket", false).unwrap();

            if !(self.rockets.len() >= invoke_fn!(self.mun_runtime, "max_rockets_amount").unwrap())
            {
                let spaceship_object: StructRef = self
                    .game_struct
                    .get::<StructRef>("spaceship")
                    .unwrap()
                    .get::<StructRef>("object")
                    .unwrap();
                let spaceship_positon = spaceship_object.get::<StructRef>("position").unwrap();

                let new_bullet: StructRef = invoke_fn!(
                    self.mun_runtime,
                    "new_rocket",
                    spaceship_positon,
                    spaceship_object.get::<f32>("angle").unwrap()
                )
                .unwrap();

                self.rockets.push(new_bullet);
            }
        }

        if self.game_struct.get::<bool>("spawn_new_asteroids").unwrap() {
            self.game_struct.set("spawn_new_asteroids", false).unwrap();

            self.asteroids = new_asteroids(&self.mun_runtime);
        }

        // Rockets update
        for index in 0..self.rockets.len() {
            let _: () = invoke_fn!(
                self.mun_runtime,
                "update_rocket",
                self.rockets[index].clone()
            )
            .unwrap();
        }
        // Delete rockets
        self.rockets
            .retain(|rocket| !rocket.get::<bool>("need_to_destroy").unwrap());

        // Asteroids update
        for index in 0..self.asteroids.len() {
            let _: () = invoke_fn!(
                self.mun_runtime,
                "update_asteroid",
                self.asteroids[index].clone()
            )
            .unwrap();
        }

        let mut new_asteroids: Vec<StructRef> = Vec::new();

        let mut asteroids = self.asteroids.clone();

        asteroids.retain(|asteroid| {
            if asteroid.get::<bool>("need_to_destroy").unwrap() {
                if asteroid.get::<u8>("size").unwrap() > 1 {
                    let asteroid_object = asteroid.get::<StructRef>("object").unwrap();

                    new_asteroids.push(
                        invoke_fn!(
                            self.mun_runtime,
                            "new_asteroid",
                            asteroid_object.get::<StructRef>("position").unwrap(),
                            thread_rng().gen_range(0.0_f32, 360.0_f32),
                            asteroid.get::<u8>("size").unwrap() - 1
                        )
                        .unwrap(),
                    );

                    new_asteroids.push(
                        invoke_fn!(
                            self.mun_runtime,
                            "new_asteroid",
                            asteroid_object.get::<StructRef>("position").unwrap(),
                            thread_rng().gen_range(0.0_f32, 360.0_f32),
                            asteroid.get::<u8>("size").unwrap() - 1
                        )
                        .unwrap(),
                    );
                }
                false
            } else {
                true
            }
        });

        asteroids.append(&mut new_asteroids);

        self.asteroids = asteroids;

        // Asteroids and rocket collision
        for rocket in self.rockets.iter_mut() {
            for asteroid in self.asteroids.iter_mut() {
                let collide: bool = invoke_fn!(
                    self.mun_runtime,
                    "object_collide",
                    rocket.get::<StructRef>("object").clone().unwrap(),
                    asteroid.get::<StructRef>("object").clone().unwrap()
                )
                .unwrap();

                if collide {
                    self.score += 1;
                    rocket.set("need_to_destroy", true).unwrap();
                    asteroid.set("need_to_destroy", true).unwrap();
                }
            }
        }

        // Asteroids and spaceship collision
        for asteroid in self.asteroids.iter() {
            let collide: bool = invoke_fn!(
                self.mun_runtime,
                "object_collide",
                self.game_struct
                    .get::<StructRef>("spaceship")
                    .unwrap()
                    .get::<StructRef>("object")
                    .unwrap(),
                asteroid.get::<StructRef>("object").clone().unwrap()
            )
            .unwrap();

            if collide {
                self.game_struct
                    .set("token", rand::thread_rng().gen::<u8>())
                    .unwrap();

                self.rockets.clear();

                self.score = 0;
            }
        }

        if self.asteroids.is_empty() {
            self.game_struct.set("spawn_new_asteroids", true).unwrap();
        }

        let _: () = invoke_fn!(
            self.mun_runtime,
            "update",
            self.game_struct.clone(),
            self.player_input.clone()
        )
        .wait();

        self.mun_runtime.borrow_mut().update();

        self.player_input = invoke_fn!(self.mun_runtime, "new_player_input").unwrap();

        Ok(())
    }
}

fn new_asteroids(mun_runtime: &Rc<RefCell<mun_runtime::Runtime>>) -> Vec<StructRef> {
    let mut asteroids = Vec::new();
    for _ in 0..invoke_fn!(mun_runtime, "initial_asteroids_amount").unwrap() {
        let position: (f32, f32) = {
            if thread_rng().gen_range(0, 1) == 0 {
                (0.0, thread_rng().gen_range(0.0, game_area_height()))
            } else {
                (0.0, thread_rng().gen_range(game_area_width(), 0.0))
            }
        };

        let asteroid_position: StructRef =
            invoke_fn!(mun_runtime, "new_vec2", position.0, position.1).unwrap();

        asteroids.push(
            invoke_fn!(
                mun_runtime,
                "new_asteroid",
                asteroid_position,
                thread_rng().gen_range(0.0_f32, 360.0_f32),
                3_u8
            )
            .unwrap(),
        );
    }
    asteroids
}

fn main() -> tetra::Result {
    let runtime = RuntimeBuilder::new("spaceship.munlib")
        .insert_fn("sin", sin as extern "C" fn(number: f32) -> f32)
        .insert_fn("cos", cos as extern "C" fn(number: f32) -> f32)
        .insert_fn("dbg", dbg as extern "C" fn(number: f32))
        .insert_fn(
            "degrees_to_radians",
            degrees_to_radians as extern "C" fn(degrees: f32) -> f32,
        )
        .insert_fn("sqrt", sqrt as extern "C" fn(value: f32) -> f32)
        .insert_fn("game_area_width", game_area_width as extern "C" fn() -> f32)
        .insert_fn(
            "game_area_height",
            game_area_height as extern "C" fn() -> f32,
        )
        .spawn()
        .expect("Failed to spawn Runtime");

    let game_struct = invoke_fn!(runtime, "new_game_struct").unwrap();

    let player_input: StructRef = invoke_fn!(runtime, "new_player_input").unwrap();

    ContextBuilder::new("Spaceship Game", 1280, 720)
        .build()?
        .run(|ctx| {
            Ok(SpaceshipGame {
                mun_runtime: runtime,
                asteroids: Vec::new(),
                rockets: Vec::new(),
                scaler: ScreenScaler::with_window_size(
                    ctx,
                    game_area_width() as i32,
                    game_area_height() as i32,
                    ScalingMode::ShowAllPixelPerfect,
                )?,
                textures: textures(ctx),
                game_struct: game_struct,
                player_input: player_input,
                font: Font::vector(ctx, "./assets/spaceship/fonts/Minimal3x5.ttf", 18.0).unwrap(),
                score: 0,
            })
        })
}
