    pub fn get_mut() -> Result<&'static mut ScoreBoard, Error> {
        unsafe {
            if let Some(scoreboard) = SCOREBOARD.as_mut() {
                Ok(scoreboard)
            } else {
                let reason: &str = "uninitialized scoreboard";
                error!("{reason}");
                Err(Error::new(ErrorCode::TryAgain, reason))
            }
        }
    }
