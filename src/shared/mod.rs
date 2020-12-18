use uom::si::{f32::{Ratio, Time}, ratio::percent, time::second};

pub struct UpdateContext {
    delta: Time
}

impl UpdateContext {
    pub fn new(delta: Time) -> UpdateContext {
        UpdateContext {
            delta
        }
    }
}

/// The delay logic gate delays the true result of a given expression by the given amount of time.
/// False results are output immediately.
pub struct DelayedTrueLogicGate {
    delay: Time,
    expression_result: bool,
    true_duration: Time
}

impl DelayedTrueLogicGate {
    pub fn new(delay: Time) -> DelayedTrueLogicGate {
        DelayedTrueLogicGate {
            delay,
            expression_result: false,
            true_duration: Time::new::<second>(0.)
        }
    }

    pub fn update(&mut self, context: &UpdateContext, expression_result: bool) {
        // We do not include the delta representing the moment before the expression_result became true.
        if self.expression_result && expression_result {
            self.true_duration += context.delta;
        } else {
            self.true_duration = Time::new::<second>(0.);
        }

        self.expression_result = expression_result;
    }

    pub fn output(&self) -> bool {
        if self.expression_result && self.delay <= self.true_duration {
            true
        } else {
            false
        }
    }
}

pub struct Engine {
    pub n2: Ratio
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            n2: Ratio::new::<percent>(0.)
        }
    }
}

#[cfg(test)]
mod delayed_true_logic_gate_tests {
    use super::*;

    #[test]
    fn when_the_expression_is_false_returns_false() {
        let mut gate = delay_logic_gate(Time::new::<second>(0.1));
        gate.update(&update_context(Time::new::<second>(0.)), false);
        gate.update(&update_context(Time::new::<second>(1.0)), false);

        assert_eq!(gate.output(), false);
    }

    #[test]
    fn when_the_expression_is_true_and_delay_hasnt_passed_returns_false() {
        let mut gate = delay_logic_gate(Time::new::<second>(10.));
        gate.update(&update_context(Time::new::<second>(0.)), false);
        gate.update(&update_context(Time::new::<second>(1.0)), false);

        assert_eq!(gate.output(), false);
    }

    #[test]
    fn when_the_expression_is_true_and_delay_has_passed_returns_true() {
        let mut gate = delay_logic_gate(Time::new::<second>(0.1));
        gate.update(&update_context(Time::new::<second>(0.)), true);
        gate.update(&update_context(Time::new::<second>(1.0)), true);

        assert_eq!(gate.output(), true);
    }

    #[test]
    fn when_the_expression_is_true_and_becomes_false_before_delay_has_passed_returns_false_once_delay_passed() {
        let mut gate = delay_logic_gate(Time::new::<second>(1.0));
        gate.update(&update_context(Time::new::<second>(0.)), true);
        gate.update(&update_context(Time::new::<second>(0.8)), true);
        gate.update(&update_context(Time::new::<second>(0.1)), false);
        gate.update(&update_context(Time::new::<second>(0.2)), false);

        assert_eq!(gate.output(), false);
    }

    #[test]
    fn does_not_include_delta_at_the_moment_of_expression_becoming_true() {
        let mut gate = delay_logic_gate(Time::new::<second>(1.0));
        gate.update(&update_context(Time::new::<second>(0.9)), true);
        gate.update(&update_context(Time::new::<second>(0.2)), true);

        assert_eq!(gate.output(), false);
    }

    fn update_context(delta: Time) -> UpdateContext {
        UpdateContext::new(delta)
    }

    fn delay_logic_gate(delay: Time) -> DelayedTrueLogicGate {
        DelayedTrueLogicGate::new(delay)
    }
}