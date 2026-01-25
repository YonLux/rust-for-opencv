use opencv::{
    core::{self, Point, Scalar, Vec3b, Vec4b, Mat},
    highgui, imgcodecs, imgproc,
    prelude::*,
};
use rand::Rng;
use std::net::UdpSocket;
use std::time::{Duration, Instant};
// ===== PNG 透明叠加函数 =====
fn overlay_png(bg: &mut Mat, fg: &Mat, x: i32, y: i32) -> opencv::Result<()> {
    let channels = fg.channels();
    let bg_size = bg.size()?;
    let fg_size = fg.size()?;

    for i in 0..fg_size.height {
        for j in 0..fg_size.width {
            let by = y + i;
            let bx = x + j;

            if by < 0 || by >= bg_size.height || bx < 0 || bx >= bg_size.width {
                continue;
            }

            if channels == 4 {
                let fg_px = *fg.at_2d::<Vec4b>(i, j)?;
                let alpha = fg_px[3] as f32 / 255.0;
                if alpha > 0.0 {
                    let bg_px = bg.at_2d_mut::<Vec3b>(by, bx)?;
                    for c in 0..3 {
                        bg_px[c] = ((1.0 - alpha) * bg_px[c] as f32 + alpha * fg_px[c] as f32) as u8;
                    }
                }
            } else {
                let fg_px = *fg.at_2d::<Vec3b>(i, j)?;
                let bg_px = bg.at_2d_mut::<Vec3b>(by, bx)?;
                *bg_px = fg_px;
            }
        }
    }
    Ok(())
}

// ===== 贪吃蛇逻辑 =====
struct SnakeGame {
    points: Vec<Point>,
    lengths: Vec<f64>,
    current_len: f64,
    allowed_len: f64,
    prev_head: Point,
    food_img: Mat,
    food_pos: Point,
    score: i32,
    game_over: bool,
}

impl SnakeGame {
    fn new(food_path: &str) -> opencv::Result<Self> {
        let food_img = imgcodecs::imread(food_path, imgcodecs::IMREAD_UNCHANGED)?;
        let mut s = Self {
            points: vec![],
            lengths: vec![],
            current_len: 0.0,
            allowed_len: 150.0,
            prev_head: Point::new(0, 0),
            food_img,
            food_pos: Point::new(0, 0),
            score: 0,
            game_over: false,
        };
        s.random_food();
        Ok(s)
    }

    fn random_food(&mut self) {
        let mut rng = rand::thread_rng();
        self.food_pos = Point::new(rng.gen_range(100..1100), rng.gen_range(100..600));
    }

    fn update(&mut self, img: &mut Mat, head: Point) -> opencv::Result<()> {
        if self.game_over {
            imgproc::put_text(img, "GAME OVER! Press 'R' to Restart", Point::new(300, 360), 
                imgproc::FONT_HERSHEY_SIMPLEX, 1.5, Scalar::new(0.0, 0.0, 255.0, 0.0), 3, imgproc::LINE_AA, false)?;
            return Ok(());
        }

        let dist = (((head.x - self.prev_head.x).pow(2) + (head.y - self.prev_head.y).pow(2)) as f64).sqrt();
        if dist < 1.0 { return Ok(()); } // 忽略微小抖动

        self.points.push(head);
        self.lengths.push(dist);
        self.current_len += dist;
        self.prev_head = head;

        while self.current_len > self.allowed_len {
            self.current_len -= self.lengths.remove(0);
            self.points.remove(0);
        }

        // 检测碰撞食物 (简化判定)
        if (head.x - self.food_pos.x).abs() < 40 && (head.y - self.food_pos.y).abs() < 40 {
            self.score += 1;
            self.allowed_len += 50.0;
            self.random_food();
        }

        // 画蛇
        for i in 1..self.points.len() {
            imgproc::line(img, self.points[i-1], self.points[i], Scalar::new(0.0, 255.0, 0.0, 0.0), 15, imgproc::LINE_AA, 0)?;
        }

        // 画食物
        let fw = self.food_img.cols();
        let fh = self.food_img.rows();
        overlay_png(img, &self.food_img, self.food_pos.x - fw/2, self.food_pos.y - fh/2)?;

        imgproc::put_text(img, &format!("Score: {}", self.score), Point::new(50, 50), 
            imgproc::FONT_HERSHEY_SIMPLEX, 1.0, Scalar::new(255.0, 255.0, 255.0, 0.0), 2, imgproc::LINE_AA, false)?;

        Ok(())
    }
}
fn main() -> opencv::Result<()> {
    // 1. 初始化 Socket (确保它在 main 的作用域内)
    let socket = UdpSocket::bind("127.0.0.1:9999").expect("无法绑定端口");
    socket.set_nonblocking(true).expect("无法设置为非阻塞模式");

    // 2. 创建画布
    // 注意：如果 core::CV_8UC3 依然报错，尝试直接写 16 (这是它的原始值)
    let mut canvas = Mat::new_rows_cols_with_default(
        720, 
        1280, 
        opencv::core::CV_8UC3, // 去掉括号
        Scalar::all(0.0)
    )?;

    // 3. 初始化游戏
    let mut game = SnakeGame::new("donut.png").expect("请确保项目根目录有 donut.png");
    
    let mut head = Point::new(640, 360);
    let mut buf = [0u8; 64];
    let frame_duration = std::time::Duration::from_millis(16); // 约 60 FPS

    println!("游戏已启动，等待 Python 发送坐标...");

    loop {
        let start_time = std::time::Instant::now();

        // 4. 接收 UDP 数据
        // 使用 while 循环排空缓冲区，只保留最新的坐标
        while let Ok((n, _)) = socket.recv_from(&mut buf) {
            if let Ok(msg) = std::str::from_utf8(&buf[..n]) {
                let parts: Vec<&str> = msg.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        head = Point::new(x, y);
                    }
                }
            }
        }

        // 5. 绘制背景 (深蓝色底 + 网格)
        imgproc::rectangle(&mut canvas, core::Rect::new(0, 0, 1280, 720), Scalar::new(40.0, 20.0, 20.0, 0.0), -1, 8, 0)?;
        
        // 绘制辅助网格线
        for i in (0..1280).step_by(80) {
            imgproc::line(&mut canvas, Point::new(i, 0), Point::new(i, 720), Scalar::new(60.0, 40.0, 40.0, 0.0), 1, imgproc::LINE_AA, 0)?;
        }
        for j in (0..720).step_by(80) {
            imgproc::line(&mut canvas, Point::new(0, j), Point::new(1280, j), Scalar::new(60.0, 40.0, 40.0, 0.0), 1, imgproc::LINE_AA, 0)?;
        }

        // 6. 更新游戏并渲染
        game.update(&mut canvas, head)?;

        // 7. 显示窗口
        highgui::imshow("Snake Game", &canvas)?;
        
        // 8. 响应按键
        let key = highgui::wait_key(1)?;
        if key == 27 { break; } // ESC 退出
        if key == 'r' as i32 { 
            game.game_over = false; 
            game.score = 0; 
            game.allowed_len = 150.0; 
            game.points.clear();
            game.lengths.clear();
            game.current_len = 0.0;
        }

        // 9. 帧率控制
        let elapsed = start_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
    Ok(())
}