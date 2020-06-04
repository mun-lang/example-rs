use ggez::{
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics::{self, DrawMode, DrawParam, FilterMode, Mesh, MeshBuilder, Rect, Text},
    nalgebra as na, Context, ContextBuilder, GameResult,
};
use mun_examples::marshal_vec2;
use mun_runtime::{invoke_fn, RetryResultExt, Runtime, RuntimeBuilder, StructRef};
use rand::Rng;
use std::{cell::RefCell, rc::Rc};

extern "C" fn rand_f32() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("Pong", "Mun Team")
        .build()
        .expect("Failed to initialize ggez");

    let runtime = RuntimeBuilder::new("pong.munlib")
        .insert_fn("rand_f32", rand_f32 as extern "C" fn() -> f32)
        .spawn()
        .expect("Failed to load munlib");

    let state: StructRef = invoke_fn!(runtime, "new_state").wait();
    let mut pong = PongGame { runtime, state };

    match event::run(&mut ctx, &mut event_loop, &mut pong) {
        Ok(_) => (),
        Err(e) => println!("Error occurred: {}", e),
    }
}

struct PongGame {
    runtime: Rc<RefCell<Runtime>>,
    state: StructRef,
}

impl EventHandler for PongGame {
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::W => {
                let mut paddle = self.state.get::<StructRef>("paddle_left").unwrap();
                paddle.set("move_up", true).unwrap();
            }
            KeyCode::S => {
                let mut paddle = self.state.get::<StructRef>("paddle_left").unwrap();
                paddle.set("move_down", true).unwrap();
            }
            KeyCode::Up => {
                let mut paddle = self.state.get::<StructRef>("paddle_right").unwrap();
                paddle.set("move_up", true).unwrap();
            }
            KeyCode::Down => {
                let mut paddle = self.state.get::<StructRef>("paddle_right").unwrap();
                paddle.set("move_down", true).unwrap();
            }
            KeyCode::Escape => {
                event::quit(ctx);
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::W => {
                let mut paddle = self.state.get::<StructRef>("paddle_left").unwrap();
                paddle.set("move_up", false).unwrap();
            }
            KeyCode::S => {
                let mut paddle = self.state.get::<StructRef>("paddle_left").unwrap();
                paddle.set("move_down", false).unwrap();
            }
            KeyCode::Up => {
                let mut paddle = self.state.get::<StructRef>("paddle_right").unwrap();
                paddle.set("move_up", false).unwrap();
            }
            KeyCode::Down => {
                let mut paddle = self.state.get::<StructRef>("paddle_right").unwrap();
                paddle.set("move_down", false).unwrap();
            }
            _ => (),
        }
    }

    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        let _: () = invoke_fn!(self.runtime, "update", self.state.clone()).wait();

        self.runtime.borrow_mut().update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, graphics::BLACK);

        let ball = self.state.get::<StructRef>("ball").unwrap();
        let paddle_left = self.state.get::<StructRef>("paddle_left").unwrap();
        let paddle_right = self.state.get::<StructRef>("paddle_right").unwrap();

        let ball_mesh = MeshBuilder::new()
            .circle(
                DrawMode::fill(),
                na::Point2::origin(),
                invoke_fn!(self.runtime, "ball_radius").unwrap(),
                invoke_fn!(self.runtime, "ball_tolerance").unwrap(),
                graphics::WHITE,
            )
            .build(ctx)?;
        draw_mesh(ctx, &ball_mesh, &ball)?;

        let paddle_mesh = MeshBuilder::new()
            .rectangle(
                DrawMode::fill(),
                bounds(
                    invoke_fn!(self.runtime, "paddle_width").unwrap(),
                    invoke_fn!(self.runtime, "paddle_height").unwrap(),
                ),
                graphics::WHITE,
            )
            .build(ctx)?;
        draw_mesh(ctx, &paddle_mesh, &paddle_left)?;
        draw_mesh(ctx, &paddle_mesh, &paddle_right)?;

        queue_score_text(
            ctx,
            &paddle_left,
            marshal_vec2(&invoke_fn!(self.runtime, "left_score_pos").unwrap()),
        );
        queue_score_text(
            ctx,
            &paddle_right,
            marshal_vec2(&invoke_fn!(self.runtime, "right_score_pos").unwrap()),
        );
        graphics::draw_queued_text(ctx, DrawParam::default(), None, FilterMode::Linear)?;

        graphics::present(ctx)?;
        Ok(())
    }
}

fn bounds(width: f32, height: f32) -> Rect {
    Rect::new(0.0, 0.0, width, height)
}

fn draw_mesh(ctx: &mut Context, mesh: &Mesh, object: &StructRef) -> GameResult {
    graphics::draw(
        ctx,
        mesh,
        (
            marshal_vec2(&object.get("pos").unwrap()),
            0.0,
            graphics::WHITE,
        ),
    )
}

fn queue_score_text(ctx: &mut Context, paddle: &StructRef, score_pos: na::Point2<f32>) {
    let score = paddle.get::<u32>("score").unwrap();
    let score_text = Text::new(score.to_string());
    graphics::queue_text(ctx, &score_text, score_pos, Some(graphics::WHITE));
}
