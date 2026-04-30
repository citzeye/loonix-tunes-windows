/* --- loonixtunesv2/src/audio/engine/abLoop.rs | A B Looper --- */

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ABLoopState {
    #[default]
    Off,
    ASet,
    Active, // A dan B sudah diset, siap nge-loop!
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ABLoop {
    state: ABLoopState,
    point_a: f64, // Posisi dalam detik (seconds)
    point_b: f64, // Posisi dalam detik (seconds)
}

impl ABLoop {
    /// Bikin instance baru, default-nya mati.
    pub fn new() -> Self {
        Self {
            state: ABLoopState::Off,
            point_a: 0.0,
            point_b: 0.0,
        }
    }

    /// Fungsi ini dipanggil pas user nge-klik tombol "A-B Repeat" di UI QML lo.
    pub fn toggle(&mut self, current_position: f64) {
        match self.state {
            ABLoopState::Off => {
                // Klik pertama: Set titik A
                self.point_a = current_position;
                self.state = ABLoopState::ASet;
                println!("A-B Repeat: Point A set at {:.2}s", self.point_a);
            }
            ABLoopState::ASet => {
                // Klik kedua: Set titik B
                if current_position > self.point_a {
                    self.point_b = current_position;
                    self.state = ABLoopState::Active;
                    println!(
                        "A-B Repeat: Active! Looping from {:.2}s to {:.2}s",
                        self.point_a, self.point_b
                    );
                } else {
                    // Kalau user nge-klik B di waktu yang lebih mundur dari A, kita reset aja
                    println!("A-B Repeat: Point B must be after Point A. Resetting.");
                    self.reset();
                }
            }
            ABLoopState::Active => {
                // Klik ketiga: Matiin A-B Repeat
                self.reset();
                println!("A-B Repeat: Off");
            }
        }
    }

    /// Paksa matiin A-B repeat (misal pas ganti lagu)
    pub fn reset(&mut self) {
        self.state = ABLoopState::Off;
        self.point_a = 0.0;
        self.point_b = 0.0;
    }

    /// Fungsi ini dipanggil terus-terusan oleh Audio Engine lo (di loop FFmpeg/Playback)
    /// Kalau dia ngereturn Some(f64), berarti engine harus nge-seek (lompat) ke detik tersebut.
    pub fn check_loop(&self, current_position: f64) -> Option<f64> {
        if self.state == ABLoopState::Active && current_position >= self.point_b {
            Some(self.point_a)
        } else {
            None
        }
    }

    // --- Getters buat ngasih info ke UI (QML) ---
    pub fn state(&self) -> ABLoopState {
        self.state
    }

    pub fn point_a(&self) -> f64 {
        self.point_a
    }

    pub fn point_b(&self) -> f64 {
        self.point_b
    }
}
