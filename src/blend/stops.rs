#[derive(Debug)]
pub struct Stop<I> {
    sample: I,
    cenith: f32,
}

impl<I> Stop<I> {
    /// Creates a  new blend stop.
    ///
    /// # Panics
    /// Panics if the given cenith is inifinite or NaN.
    pub fn new(cenith: f32, sample: I) -> Self {
        if cenith.is_infinite() {
            panic!(
                "Some blend stop cenith was NaN/Infinity during construction: {:?}",
                cenith
            );
        }

        Self { sample, cenith }
    }

    pub fn sample(&self) -> &I {
        &self.sample
    }

    pub fn cenith(&self) -> f32 {
        self.cenith
    }
}

#[derive(Debug)]
pub struct Stops<I> {
    /// Blend stops sorted by cenith
    stops: Vec<Stop<I>>,
}

impl<I> Stops<I> {
    /// Creates blend stops from a non-empty iterator.
    ///
    /// # Panics
    /// Panics if the given iterator does not yield at least one element.
    pub fn new(stops: impl IntoIterator<Item = Stop<I>>) -> Self {
        let mut stops = stops.into_iter().collect::<Vec<_>>();

        if stops.is_empty() {
            panic!(
                "Tried to create blend stops with an empty iterator of stops, which is undefined"
            );
        }

        if stops.iter().any(|s| s.cenith.is_infinite()) {
            let ceniths = stops.iter().map(|s| s.cenith).collect::<Vec<_>>();
            panic!(
                "Some ceniths were NaN/Infinity during guided blend stop construction: {:?}",
                ceniths
            );
        }

        // Sort for fast lookup of cenith before and after
        stops.sort_by(|a, b| {
            a.cenith.partial_cmp(&b.cenith).unwrap() // NaN or infinite would have panicked before, comparison is safe
        });

        Self { stops }
    }

    pub fn stops_before_after(&self, guide: f32) -> (&Stop<I>, &Stop<I>) {
        let mut stop_iter = self.stops.iter();
        let mut last_stop = stop_iter.next().unwrap(); // always at least one

        while let Some(stop) = stop_iter.next() {
            if last_stop.cenith <= guide && stop.cenith > guide {
                return (last_stop, stop);
            }
            last_stop = stop;
        }

        // If only one stop specified, it is the last stop and returned twice.
        // If no stop has a cenith greater than the guide,
        // repeat the stop with the highest cenith
        (last_stop, last_stop)
    }
}
