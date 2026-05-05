use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use veila_common::config::{BackgroundConfig, BackgroundSlideshowOrder};

pub(crate) struct BackgroundSlideshow {
    paths: Vec<PathBuf>,
    order: BackgroundSlideshowOrder,
    sequence: Vec<usize>,
    position: usize,
    change_interval: Duration,
    next_change_at: Option<Instant>,
}

impl BackgroundSlideshow {
    pub(crate) fn load(background: &BackgroundConfig) -> Option<Self> {
        if !background.slideshow_enabled() {
            return None;
        }

        let slideshow = background.slideshow.as_ref()?;
        let paths = match background.resolved_slideshow_paths() {
            Ok(paths) => paths,
            Err(error) => {
                tracing::warn!("failed to resolve slideshow wallpaper sources: {error:#}");
                return None;
            }
        };

        if paths.is_empty() {
            return None;
        }

        let order = slideshow.order;
        let mut sequence: Vec<_> = (0..paths.len()).collect();
        if order == BackgroundSlideshowOrder::Random && sequence.len() > 1 {
            shuffle_tail(&mut sequence);
        }

        let next_change_at =
            (paths.len() > 1).then(|| Instant::now() + slideshow.change_interval());

        Some(Self {
            paths,
            order,
            sequence,
            position: 0,
            change_interval: slideshow.change_interval(),
            next_change_at,
        })
    }

    pub(crate) fn current_path(&self) -> &Path {
        &self.paths[self.sequence[self.position]]
    }

    pub(crate) fn len(&self) -> usize {
        self.paths.len()
    }

    pub(crate) fn next_due_in(&self, now: Instant) -> Option<Duration> {
        self.next_change_at
            .map(|deadline| deadline.saturating_duration_since(now))
    }

    pub(crate) fn advance(&mut self, now: Instant) -> Option<PathBuf> {
        let next_change_at = self.next_change_at?;
        if now < next_change_at || self.paths.len() < 2 {
            return None;
        }

        self.position += 1;
        if self.position >= self.sequence.len() {
            self.reset_sequence();
        }

        self.next_change_at = Some(now + self.change_interval);
        Some(self.current_path().to_path_buf())
    }

    pub(crate) fn next_preload_path(&self) -> Option<PathBuf> {
        if self.paths.len() < 2 {
            return None;
        }

        let mut next_position = self.position + 1;
        if next_position >= self.sequence.len() {
            next_position = 0;
        }

        Some(self.paths[self.sequence[next_position]].clone())
    }

    fn reset_sequence(&mut self) {
        self.position = 0;
        if self.order == BackgroundSlideshowOrder::Sequence || self.sequence.len() < 2 {
            return;
        }

        let previous_last = *self.sequence.last().unwrap_or(&0);
        self.sequence = (0..self.paths.len()).collect();
        shuffle_tail(&mut self.sequence);
        if self.sequence[0] == previous_last {
            self.sequence.swap(0, 1);
        }
    }
}

fn shuffle_tail(sequence: &mut [usize]) {
    let mut state = shuffle_seed();
    for index in (1..sequence.len()).rev() {
        let offset = next_u64(&mut state) as usize % index;
        sequence.swap(index, 1 + offset);
    }
}

fn shuffle_seed() -> u64 {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    seed ^ 0x9E37_79B9_7F4A_7C15
}

fn next_u64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        time::{Duration, Instant},
    };

    use super::{BackgroundSlideshow, shuffle_tail};

    #[test]
    fn random_shuffle_keeps_first_item_stable() {
        let mut sequence = vec![0, 1, 2, 3];
        shuffle_tail(&mut sequence);

        assert_eq!(sequence[0], 0);
        assert_eq!(sequence.len(), 4);
        assert!(sequence.contains(&1));
        assert!(sequence.contains(&2));
        assert!(sequence.contains(&3));
    }

    #[test]
    fn next_preload_wraps_to_start() {
        let mut slideshow = BackgroundSlideshow {
            paths: vec!["/tmp/one.jpg".into(), "/tmp/two.jpg".into()],
            order: veila_common::config::BackgroundSlideshowOrder::Sequence,
            sequence: vec![0, 1],
            position: 1,
            change_interval: Duration::from_secs(5),
            next_change_at: Some(Instant::now()),
        };

        assert_eq!(
            slideshow.next_preload_path().as_deref(),
            Some(Path::new("/tmp/one.jpg"))
        );

        slideshow.reset_sequence();
        assert_eq!(slideshow.position, 0);
    }
}
