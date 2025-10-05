
use anyhow::{bail, Result};
use std::collections::HashMap;
use crate::value::Value;
use crate::env::Env;
use crate::parser::Expr;

impl Env {
    pub fn import_sdl(&mut self) -> Value {
        let mut obj = HashMap::new();
        #[cfg(not(feature = "sdl3"))]
        {
            // Provide a one-time notice that this build is headless for SDL.
            if !self.modules.contains_key("__sdl_notice_shown") {
                eprintln!("[TONG][SDL] Built without 'sdl3' feature: using headless shim (no real window). Rebuild with --features sdl3 for graphics.");
                self.modules
                    .insert("__sdl_notice_shown".to_string(), Value::Bool(true));
            }
        }
        // constants
        obj.insert("K_ESCAPE".to_string(), Value::Int(27));
        obj.insert("K_Q".to_string(), Value::Int(81));
        obj.insert("K_W".to_string(), Value::Int(87));
        obj.insert("K_S".to_string(), Value::Int(83));
        obj.insert("K_UP".to_string(), Value::Int(1000));
        obj.insert("K_DOWN".to_string(), Value::Int(1001));
        // functions (method names map to builtin function identifiers)
        obj.insert("init".into(), Value::FuncRef("sdl_init".into()));
        obj.insert(
            "create_window".into(),
            Value::FuncRef("sdl_create_window".into()),
        );
        obj.insert(
            "create_renderer".into(),
            Value::FuncRef("sdl_create_renderer".into()),
        );
        obj.insert(
            "set_draw_color".into(),
            Value::FuncRef("sdl_set_draw_color".into()),
        );
        obj.insert("clear".into(), Value::FuncRef("sdl_clear".into()));
        obj.insert("fill_rect".into(), Value::FuncRef("sdl_fill_rect".into()));
        obj.insert("present".into(), Value::FuncRef("sdl_present".into()));
        obj.insert("delay".into(), Value::FuncRef("sdl_delay".into()));
        obj.insert("poll_quit".into(), Value::FuncRef("sdl_poll_quit".into()));
        obj.insert("key_down".into(), Value::FuncRef("sdl_key_down".into()));
        obj.insert(
            "destroy_renderer".into(),
            Value::FuncRef("sdl_destroy_renderer".into()),
        );
        obj.insert(
            "destroy_window".into(),
            Value::FuncRef("sdl_destroy_window".into()),
        );
        obj.insert("quit".into(), Value::FuncRef("sdl_quit".into()));
        Value::Object(obj)
    }

    pub fn call_sdl_builtin(&mut self, name: &str, args: Vec<Expr>) -> Result<Value> {
        #[cfg(feature = "sdl3")]
        {
            self.call_sdl_builtin_real(name, args)
        }
        #[cfg(not(feature = "sdl3"))]
        {
            match name {
                "sdl_init" => Ok(Value::Int(0)),
                "sdl_create_window" => Ok(Value::Int(1)),
                "sdl_create_renderer" => Ok(Value::Int(1)),
                "sdl_set_draw_color" => Ok(Value::Int(0)),
                "sdl_clear" => Ok(Value::Int(0)),
                "sdl_fill_rect" => Ok(Value::Int(0)),
                "sdl_present" => Ok(Value::Int(0)),
                "sdl_delay" => {
                    // Simulate ~60 FPS by increasing frame count; no sleeping for CI speed
                    let _ = args; // ignore actual ms
                    self.sdl_frame += 1;
                    Ok(Value::Int(0))
                }
                "sdl_poll_quit" => {
                    let quit = self.sdl_frame >= 300; // auto-quit after ~300 frames
                    Ok(Value::Bool(quit))
                }
                "sdl_key_down" => Ok(Value::Bool(false)),
                "sdl_destroy_renderer" => Ok(Value::Int(0)),
                "sdl_destroy_window" => Ok(Value::Int(0)),
                "sdl_quit" => Ok(Value::Int(0)),
                other => bail!("unknown SDL builtin {}", other),
            }
        }
    }
}

#[cfg(feature = "sdl3")]
struct SdlState {
    _sdl: sdl3::Sdl,
    video: sdl3::VideoSubsystem,
    window: Option<sdl3::video::Window>,
    canvas: Option<sdl3::render::Canvas<sdl3::video::Window>>,
    events: sdl3::EventPump,
    draw_color: (u8, u8, u8, u8),
}

#[cfg(feature = "sdl3")]
impl Env {
    fn sdl_state_mut(&mut self) -> Result<&mut SdlState> {
        if self.sdl.is_none() {
            let sdl = sdl3::init().map_err(|e| anyhow!(e))?;
            let video = sdl.video().map_err(|e| anyhow!(e))?;
            let events = sdl.event_pump().map_err(|e| anyhow!(e))?;
            self.sdl = Some(SdlState {
                _sdl: sdl,
                video,
                window: None,
                canvas: None,
                events,
                draw_color: (0, 0, 0, 255),
            });
        }
        Ok(self.sdl.as_mut().unwrap())
    }

    fn call_sdl_builtin_real(&mut self, name: &str, args: Vec<Expr>) -> Result<Value> {
        use sdl3::{event::Event, keyboard::Scancode, pixels::Color, rect::Rect};
        match name {
            "sdl_init" => {
                let _ = self.sdl_state_mut()?; // ensure initialized
                Ok(Value::Int(0))
            }
            "sdl_create_window" => {
                // evaluate arguments first to avoid borrow conflicts
                let title = match args.first().map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Str(s))) => s,
                    _ => "TONG".to_string(),
                };
                let w = match args.get(1).map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Int(i))) => i as u32,
                    _ => 800,
                };
                let h = match args.get(2).map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Int(i))) => i as u32,
                    _ => 600,
                };
                let state = self.sdl_state_mut()?;
                let window = state
                    .video
                    .window(&title, w, h)
                    .position_centered()
                    .build()
                    .map_err(|e| anyhow!(e))?;
                state.window = Some(window);
                Ok(Value::Int(1))
            }
            "sdl_create_renderer" => {
                let state = self.sdl_state_mut()?;
                let window = state
                    .window
                    .take()
                    .ok_or_else(|| anyhow!("create_renderer: window not created"))?;
                // sdl3 API: into_canvas() returns a Canvas directly (no builder chain)
                let canvas = window.into_canvas();
                state.canvas = Some(canvas);
                Ok(Value::Int(1))
            }
            "sdl_set_draw_color" => {
                let (r, g, b, a) = (
                    self.eval_expr(args[1].clone())?.as_int_u8()?,
                    self.eval_expr(args[2].clone())?.as_int_u8()?,
                    self.eval_expr(args[3].clone())?.as_int_u8()?,
                    self.eval_expr(args[4].clone())?.as_int_u8()?,
                );
                let state = self.sdl_state_mut()?;
                let canvas = state
                    .canvas
                    .as_mut()
                    .ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.set_draw_color(Color::RGBA(r, g, b, a));
                state.draw_color = (r, g, b, a);
                Ok(Value::Int(0))
            }
            "sdl_clear" => {
                let state = self.sdl_state_mut()?;
                let canvas = state
                    .canvas
                    .as_mut()
                    .ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.clear();
                Ok(Value::Int(0))
            }
            "sdl_fill_rect" => {
                // args: (ren, x,y,w,h, r,g,b,a)
                let x = self.eval_expr(args[1].clone())?.as_int_i32()?;
                let y = self.eval_expr(args[2].clone())?.as_int_i32()?;
                let w = self.eval_expr(args[3].clone())?.as_int_u32()?;
                let h = self.eval_expr(args[4].clone())?.as_int_u32()?;
                let (r, g, b, a) = (
                    self.eval_expr(args[5].clone())?.as_int_u8()?,
                    self.eval_expr(args[6].clone())?.as_int_u8()?,
                    self.eval_expr(args[7].clone())?.as_int_u8()?,
                    self.eval_expr(args[8].clone())?.as_int_u8()?,
                );
                let state = self.sdl_state_mut()?;
                let canvas = state
                    .canvas
                    .as_mut()
                    .ok_or_else(|| anyhow!("renderer not created"))?;
                let prev = state.draw_color;
                canvas.set_draw_color(Color::RGBA(r, g, b, a));
                canvas.fill_rect(Rect::new(x, y, w, h)).ok();
                canvas.set_draw_color(Color::RGBA(prev.0, prev.1, prev.2, prev.3));
                Ok(Value::Int(0))
            }
            "sdl_present" => {
                let state = self.sdl_state_mut()?;
                let canvas = state
                    .canvas
                    .as_mut()
                    .ok_or_else(|| anyhow!("renderer not created"))?;
                canvas.present();
                Ok(Value::Int(0))
            }
            "sdl_delay" => {
                let ms = match args.first().map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Int(i))) => i,
                    _ => 16,
                };
                std::thread::sleep(Duration::from_millis(ms as u64));
                Ok(Value::Int(0))
            }
            "sdl_poll_quit" => {
                let state = self.sdl_state_mut()?;
                let mut quit = false;
                for event in state.events.poll_iter() {
                    if let Event::Quit { .. } = event {
                        quit = true;
                        break;
                    }
                }
                Ok(Value::Bool(quit))
            }
            "sdl_key_down" => {
                let code = match args.first().map(|e| self.eval_expr(e.clone())) {
                    Some(Ok(Value::Int(i))) => i,
                    _ => 0,
                };
                let state = self.sdl_state_mut()?;
                let kb = state.events.keyboard_state();
                let pressed = match code {
                    27 => kb.is_scancode_pressed(Scancode::Escape),
                    81 => kb.is_scancode_pressed(Scancode::Q),
                    87 => kb.is_scancode_pressed(Scancode::W),
                    83 => kb.is_scancode_pressed(Scancode::S),
                    1000 => kb.is_scancode_pressed(Scancode::Up),
                    1001 => kb.is_scancode_pressed(Scancode::Down),
                    _ => false,
                };
                Ok(Value::Bool(pressed))
            }
            "sdl_destroy_renderer" => {
                let state = self.sdl_state_mut()?;
                let _ = args; // ignore handle
                state.canvas = None;
                Ok(Value::Int(0))
            }
            "sdl_destroy_window" => {
                let state = self.sdl_state_mut()?;
                let _ = args; // ignore handle
                state.window = None;
                Ok(Value::Int(0))
            }
            "sdl_quit" => {
                // Drop everything
                if let Some(st) = self.sdl.as_mut() {
                    st.window = None;
                }
                Ok(Value::Int(0))
            }
            other => bail!("unknown SDL builtin {}", other),
        }
    }
}