pub struct Stats {
    pub pps_history: Vec<(f64, f64)>, // (time, pps)
    pub bps_history: Vec<(f64, f64)>, // (time, bps)
    pub max_points: usize,
}

impl Stats {
    pub fn new(max_points: usize) -> Self {
        Self {
            pps_history: Vec::new(),
            bps_history: Vec::new(),
            max_points,
        }
    }

    pub fn push(&mut self, t: f64, pps: f64, bps: f64) {
        self.pps_history.push((t, pps));
        self.bps_history.push((t, bps));

        if self.pps_history.len() > self.max_points {
            self.pps_history.remove(0);
            self.bps_history.remove(0);
        }
    }
}
