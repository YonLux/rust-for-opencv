use opencv::{
    core::{Point, Scalar, Vec3b, Vec4b},
    highgui, imgcodecs, imgproc,
    prelude::*,
    videoio,
};
use rand::Rng;
use std::net::UdpSocket;

// ===== PNG 透明叠加 =====
fn overlay_png(bg: &mut Mat, fg: &Mat, x: i32, y: i32) -> opencv::Result<()> {
    let channels = fg.channels();

    for i in 0..fg.rows() {
        for j in 0..fg.cols() {
            let by = y + i;
            let bx = x + j;

            if by < 0 || by >= bg.rows() || bx < 0 || bx >= bg.cols() {
                continue;
            }

            if channels == 4 {
                // ---- BGRA ----
                let fg_px = *fg.at_2d::<Vec4b>(i, j)?;
                let alpha = fg_px[3] as f32 / 255.0;
                if alpha == 0.0 {
                    continue;
                }

                let bg_px = bg.at_2d_mut::<Vec3b>(by, bx)?;
                for c in 0..3 {
                    bg_px[c] =
                        ((1.0 - alpha) * bg_px[c] as f32 +
                         alpha * fg_px[c] as f32) as u8;
                }
            } else if channels == 3 {
                // ---- BGR ----
                let fg_px = *fg.at_2d::<Vec3b>(i, j)?;
                let bg_px = bg.at_2d_mut::<Vec3b>(by, bx)?;
                bg_px[0] = fg_px[0];
                bg_px[1] = fg_px[1];
                bg_px[2] = fg_px[2];
            }
        }
    }
    Ok(())
}


// ===== 贪吃蛇 =====
struct SnakeGame {
    points: Vec<Point>,
    lengths: Vec<f64>,
    current_len: f64,
    allowed_len: f64,
    prev_head: Point,

    food_img: Mat,
    food_w: i32,
    food_h: i32,
    food_pos: Point,

    score: i32,
    game_over: bool,
}

impl SnakeGame {
    fn new(food: &str) -> opencv::Result<Self> {
        let food_img = imgcodecs::imread(food, imgcodecs::IMREAD_UNCHANGED)?;
        let sz = food_img.size()?;

        let mut s = Self {
            points: vec![],
            lengths: vec![],
            current_len: 0.0,
            allowed_len: 150.0,
            prev_head: Point::new(0, 0),

            food_img,
            food_w: sz.width,
            food_h: sz.height,
            food_pos: Point::new(0, 0),

            score: 0,
            game_over: false,
        };
        s.random_food();
        Ok(s)
    }

    fn random_food(&mut self) {
        let mut rng = rand::thread_rng();
        self.food_pos = Point::new(
            rng.gen_range(100..1000),
            rng.gen_range(100..600),
        );
    }

    fn update(&mut self, img: &mut Mat, head: Point) -> opencv::Result<()> {
        if self.game_over {
            imgproc::put_text(
                img,
                "GAME OVER",
                Point::new(300, 400),
                imgproc::FONT_HERSHEY_SIMPLEX,
                2.0,
                Scalar::new(0.0, 0.0, 255.0, 0.0),
                5,
                imgproc::LINE_AA,
                false,
            )?;
            return Ok(());
        }

        let dx = head.x - self.prev_head.x;
        let dy = head.y - self.prev_head.y;
        let dist = ((dx * dx + dy * dy) as f64).sqrt();

        self.points.push(head);
        self.lengths.push(dist);
        self.current_len += dist;
        self.prev_head = head;

        while self.current_len > self.allowed_len {
            self.current_len -= self.lengths[0];
            self.lengths.remove(0);
            self.points.remove(0);
        }

        // 吃食物
        if (self.food_pos.x - self.food_w / 2..self.food_pos.x + self.food_w / 2).contains(&head.x)
            && (self.food_pos.y - self.food_h / 2..self.food_pos.y + self.food_h / 2).contains(&head.y)
        {
            self.random_food();
            self.allowed_len += 50.0;
            self.score += 1;
        }

        // 画蛇
        for i in 1..self.points.len() {
            imgproc::line(
                img,
                self.points[i - 1],
                self.points[i],
                Scalar::new(0.0, 0.0, 255.0, 0.0),
                20,
                imgproc::LINE_AA,
                0,
            )?;
        }

        if let Some(&h) = self.points.last() {
            imgproc::circle(
                img,
                h,
                20,
                Scalar::new(200.0, 0.0, 200.0, 0.0),
                -1,
                imgproc::LINE_AA,
                0,
            )?;
        }

        // 食物
        overlay_png(
            img,
            &self.food_img,
            self.food_pos.x - self.food_w / 2,
            self.food_pos.y - self.food_h / 2,
        )?;

        imgproc::put_text(
            img,
            &format!("Score: {}", self.score),
            Point::new(50, 80),
            imgproc::FONT_HERSHEY_SIMPLEX,
            1.5,
            Scalar::new(0.0, 255.0, 0.0, 0.0),
            3,
            imgproc::LINE_AA,
            false,
        )?;

        Ok(())
    }
}

fn main() -> opencv::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:9999").unwrap();
    socket.set_nonblocking(true).unwrap();

    let mut cap = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    cap.set(videoio::CAP_PROP_FRAME_WIDTH, 1280.0)?;
    cap.set(videoio::CAP_PROP_FRAME_HEIGHT, 720.0)?;

    let mut game = SnakeGame::new("donut.png")?;
    highgui::named_window("Snake", highgui::WINDOW_AUTOSIZE)?;

    let mut buf = [0u8; 32];
    let mut head = Point::new(640, 360);

    loop {
        if let Ok((n, _)) = socket.recv_from(&mut buf) {
            if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                let mut it = s.split(',');
                if let (Some(x), Some(y)) = (it.next(), it.next()) {
                    head = Point::new(x.parse().unwrap(), y.parse().unwrap());
                }
            }
        }

        let mut frame = Mat::default();
        cap.read(&mut frame)?;
        if frame.empty() {
            continue;
        }

        game.update(&mut frame, head)?;
        highgui::imshow("Snake", &frame)?;

        let key = highgui::wait_key(1)?;
        if key == 27 {
            break;
        }
        if key == 'r' as i32 {
            game.game_over = false;
        }
    }
    Ok(())
}
