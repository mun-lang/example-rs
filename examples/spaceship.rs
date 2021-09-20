#[cfg(not(feature = "spaceship"))]
fn main() {
    panic!("Spaceship needs feature spaceship enabled: `cargo r --example spaceship --features spaceship`")
}

#[cfg(feature = "spaceship")]
mod spaceship {
    use tetra::graphics::{self, Color, DrawParams, Texture};
    use tetra::math::Vec2;
    use tetra::{Context, ContextBuilder, State};

    use tetra::graphics::scaling::{ScalingMode, ScreenScaler};

    use tetra::graphics::text::{Font, Text};

    use tetra::input::{self, Key};

    use mun_runtime::{invoke_fn, RootedStruct, RuntimeBuilder, StructRef};

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
        asteroids: Vec<RootedStruct>,
        rockets: Vec<RootedStruct>,
        textures: [(Texture, Vec2<f32>); 5],
        scaler: ScreenScaler,
        game_struct: RootedStruct,
        player_input: RootedStruct,
        font: Font,
        score: u8,
    }

    impl State for SpaceshipGame {
        fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
            graphics::set_canvas(ctx, self.scaler.canvas());

            graphics::clear(ctx, Color::BLACK);

            let spaceship_object: StructRef = self
                .game_struct
                .by_ref()
                .get::<StructRef>("spaceship")
                .unwrap()
                .get::<StructRef>("object")
                .unwrap();
            let spaceship_object_position = spaceship_object.get::<StructRef>("position").unwrap();

            // Draw rockets
            for rocket in self.rockets.iter() {
                let rocket_object = rocket.by_ref().get::<StructRef>("object").unwrap();
                let rocket_position = rocket_object.get::<StructRef>("position").unwrap();

                self.textures[1].0.draw(
                    ctx,
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
            self.textures[0].0.draw(
                ctx,
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
                let asteroid_object = asteroid.by_ref().get::<StructRef>("object").unwrap();
                let asteroid_position = asteroid_object.get::<StructRef>("position").unwrap();
                let asteroid_size: usize = asteroid.by_ref().get::<u8>("size").unwrap().into();

                self.textures[asteroid_size + 1].0.draw(
                    ctx,
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

            self.scaler.draw(ctx);

            // Draw score
            &Text::new(format!("Score {}", self.score), self.font.clone())
                .draw(ctx, Vec2::new(10., 10.));

            Ok(())
        }

        fn update(&mut self, ctx: &mut Context) -> tetra::Result {
            // Collect input to pass it into mun runtime
            let player_input = self.player_input.by_ref().clone();
            let player_input = player_input.root(self.mun_runtime.clone());
            let mut player_input = player_input.by_ref().clone();

            if input::is_key_down(ctx, Key::Left) {
                player_input.set("left", true).unwrap();
            }
            if input::is_key_down(ctx, Key::Right) {
                player_input.set("right", true).unwrap();
            }
            if input::is_key_down(ctx, Key::Up) {
                player_input.set("up", true).unwrap();
            }
            if input::is_key_down(ctx, Key::Z) {
                player_input.set("shoot", true).unwrap();
            }

            let runtime_ref = self.mun_runtime.borrow();
            let mut game_struct = self.game_struct.by_ref().clone();

            if game_struct.get::<bool>("spawn_new_rocket").unwrap() {
                game_struct.set("spawn_new_rocket", false).unwrap();

                if !(self.rockets.len() >= invoke_fn!(&runtime_ref, "max_rockets_amount").unwrap())
                {
                    let spaceship_object: StructRef = game_struct
                        .get::<StructRef>("spaceship")
                        .unwrap()
                        .get::<StructRef>("object")
                        .unwrap();
                    let spaceship_positon = spaceship_object.get::<StructRef>("position").unwrap();

                    let new_bullet: StructRef = invoke_fn!(
                        &runtime_ref,
                        "new_rocket",
                        spaceship_positon,
                        spaceship_object.get::<f32>("angle").unwrap()
                    )
                    .unwrap();

                    self.rockets.push(new_bullet.root(self.mun_runtime.clone()));
                }
            }

            if game_struct.get::<bool>("spawn_new_asteroids").unwrap() {
                game_struct.set("spawn_new_asteroids", false).unwrap();

                self.asteroids = new_asteroids(&self.mun_runtime);
            }

            // Rockets update
            for index in 0..self.rockets.len() {
                let _: () = invoke_fn!(
                    &runtime_ref,
                    "update_rocket",
                    self.rockets[index].by_ref().clone()
                )
                .unwrap();
            }
            // Delete rockets
            self.rockets
                .retain(|rocket| !rocket.by_ref().get::<bool>("need_to_destroy").unwrap());

            // Asteroids update
            for index in 0..self.asteroids.len() {
                let _: () = invoke_fn!(
                    &runtime_ref,
                    "update_asteroid",
                    self.asteroids[index].by_ref().clone()
                )
                .unwrap();
            }

            let mut new_asteroids: Vec<StructRef> = Vec::new();

            let mut asteroids: Vec<RootedStruct> = self
                .asteroids
                .iter()
                .map(|asteroid| {
                    let asteroid = asteroid.by_ref().clone();
                    asteroid.root(self.mun_runtime.clone())
                })
                .collect();

            asteroids.retain(|asteroid| {
                if asteroid.by_ref().get::<bool>("need_to_destroy").unwrap() {
                    if asteroid.by_ref().get::<u8>("size").unwrap() > 1 {
                        let asteroid_object = asteroid.by_ref().get::<StructRef>("object").unwrap();

                        new_asteroids.push(
                            invoke_fn!(
                                &runtime_ref,
                                "new_asteroid",
                                asteroid_object.get::<StructRef>("position").unwrap(),
                                thread_rng().gen_range(0.0_f32..360.0_f32),
                                asteroid.by_ref().get::<u8>("size").unwrap() - 1
                            )
                            .unwrap(),
                        );

                        new_asteroids.push(
                            invoke_fn!(
                                &runtime_ref,
                                "new_asteroid",
                                asteroid_object.get::<StructRef>("position").unwrap(),
                                thread_rng().gen_range(0.0_f32..360.0_f32),
                                asteroid.by_ref().get::<u8>("size").unwrap() - 1
                            )
                            .unwrap(),
                        );
                    }
                    false
                } else {
                    true
                }
            });

            asteroids.append(
                &mut new_asteroids
                    .into_iter()
                    .map(|elem| elem.root(self.mun_runtime.clone()))
                    .collect(),
            );

            self.asteroids = asteroids;

            // Asteroids and rocket collision
            for rocket in self.rockets.iter_mut() {
                for asteroid in self.asteroids.iter_mut() {
                    let mut rocket = rocket.by_ref().clone();
                    let mut asteroid = asteroid.by_ref().clone();

                    let collide: bool = invoke_fn!(
                        &runtime_ref,
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
                    runtime_ref,
                    "object_collide",
                    game_struct
                        .get::<StructRef>("spaceship")
                        .unwrap()
                        .get::<StructRef>("object")
                        .unwrap(),
                    asteroid
                        .by_ref()
                        .get::<StructRef>("object")
                        .clone()
                        .unwrap()
                )
                .unwrap();

                if collide {
                    game_struct
                        .set("token", rand::thread_rng().gen::<u8>())
                        .unwrap();

                    self.rockets.clear();

                    self.score = 0;
                }
            }

            if self.asteroids.is_empty() {
                game_struct.set("spawn_new_asteroids", true).unwrap();
            }

            let _: () =
                invoke_fn!(&runtime_ref, "update", game_struct, player_input.clone()).unwrap();

            // Drop shared refernce to the runtime so we can borrow it mutably
            drop(runtime_ref);
            self.mun_runtime.borrow_mut().update();

            let runtime_ref = self.mun_runtime.borrow();
            let new_player_input: StructRef = invoke_fn!(&runtime_ref, "new_player_input").unwrap();
            self.player_input = new_player_input.root(self.mun_runtime.clone());

            Ok(())
        }
    }

    fn new_asteroids(mun_runtime: &Rc<RefCell<mun_runtime::Runtime>>) -> Vec<RootedStruct> {
        let runtime_ref = mun_runtime.borrow();
        let mut asteroids = Vec::new();
        for _ in 0..invoke_fn!(&runtime_ref, "initial_asteroids_amount").unwrap() {
            let position: (f32, f32) = {
                if thread_rng().gen_range(0..1) == 0 {
                    (0.0, thread_rng().gen_range(0.0..game_area_height()))
                } else {
                    (0.0, thread_rng().gen_range(game_area_width()..0.0))
                }
            };

            let asteroid_position: StructRef =
                invoke_fn!(&runtime_ref, "new_vec2", position.0, position.1).unwrap();

            let asteroid: StructRef = invoke_fn!(
                &runtime_ref,
                "new_asteroid",
                asteroid_position,
                thread_rng().gen_range(0.0_f32..360.0_f32),
                3_u8
            )
            .unwrap();
            asteroids.push(asteroid.root(mun_runtime.clone()));
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

        let game_struct = {
            let runtime_ref = runtime.borrow();
            let game_struct: StructRef = invoke_fn!(&runtime_ref, "new_game_struct").unwrap();
            game_struct.root(runtime.clone())
        };

        let player_input = {
            let runtime_ref = runtime.borrow();
            let player_input: StructRef = invoke_fn!(&runtime_ref, "new_player_input").unwrap();
            player_input.root(runtime.clone())
        };

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
                    font: Font::vector(ctx, "./assets/spaceship/fonts/Minimal3x5.ttf", 18.0)
                        .unwrap(),
                    score: 0,
                })
            })
    }
}
