use uom::si::{electric_current::ampere, electric_potential::volt, f32::{Frequency, ElectricPotential, ElectricCurrent, Ratio}, frequency::hertz, ratio::percent};

#[derive(Clone, Copy, Debug, PartialEq)]
enum PowerSource {
    None,
    EngineGenerator(u8),
    ApuGenerator,
    External
}

/// Represents a type of electric current.
#[derive(Clone, Copy, Debug)]
enum Current {
    Alternating(PowerSource, Frequency, ElectricPotential, ElectricCurrent),
    Direct(PowerSource, ElectricPotential, ElectricCurrent),
    None
}

impl Current {
    fn is_alternating(self) -> bool {
        if let Current::Alternating(..) = self { true } else { false }
    }

    fn is_direct(self) -> bool {
        if let Current::Direct(..) = self { true } else { false }
    }

    fn is_none(self) -> bool {
        if let Current::None = self { true } else { false }
    }

    fn get_source(self) -> PowerSource {
        match self {
            Current::Alternating(source, ..) => source,
            Current::Direct(source, ..) => source,
            _ => PowerSource::None
        }
    }
}

trait PowerConductor {
    fn output(&self) -> Current;
}

trait Powerable {
    fn powered_by<T: PowerConductor + ?Sized>(&mut self, sources: Vec<&T>) {
        self.set_input(sources.iter().find_map(|x| {
            let output = x.output();
            match output {
                Current::None => None,
                _ => Some(output)
            }
        }).unwrap_or(Current::None));
    }

    fn set_input(&mut self, current: Current);
}

/// Represents the state of a contactor.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ContactorState {
    Open,
    Closed
}

/// Represents a contactor in a electrical power circuit.
#[derive(Debug)]
struct Contactor {
    state: ContactorState,
    input: Current,
}

impl Contactor {
    fn new() -> Contactor {
        Contactor {
            state: ContactorState::Open,
            input: Current::None,
        }
    }

    fn toggle(&mut self, should_be_closed: bool) {
        self.state = match self.state {
            ContactorState::Open if should_be_closed => ContactorState::Closed,
            ContactorState::Closed if !should_be_closed => ContactorState::Open,
            _ => self.state
        };
    }
}

impl Powerable for Contactor {
    fn set_input(&mut self, current: Current) {
        self.input = current;
    }
}

impl PowerConductor for Contactor {
    fn output(&self) -> Current {
        if let ContactorState::Closed = self.state {
            self.input
        } else {
            Current::None
        }
    }
}

struct A320ElectricalCircuit {
    engine_1_gen: EngineGenerator,
    engine_1_contactor: Contactor,
    engine_2_gen: EngineGenerator,
    engine_2_contactor: Contactor,
    apu_gen: ApuGenerator,
    ext_pwr: ExternalPowerSource,
    ac_bus_1: ElectricalBus,
    ac_bus_2: ElectricalBus,
}

impl A320ElectricalCircuit {
    fn new() -> A320ElectricalCircuit {
        A320ElectricalCircuit {
            engine_1_gen: EngineGenerator::new(1),
            engine_1_contactor: Contactor::new(),
            engine_2_gen: EngineGenerator::new(2),
            engine_2_contactor: Contactor::new(),
            apu_gen: ApuGenerator::new(),
            ext_pwr: ExternalPowerSource::new(),
            ac_bus_1: ElectricalBus::new(),
            ac_bus_2: ElectricalBus::new()
        }
    }

    fn update(&mut self, engine1: &Engine, engine2: &Engine, apu: &AuxiliaryPowerUnit) {
        self.engine_1_gen.update(engine1);
        let gen_1_has_output = self.engine_1_gen.output().is_alternating();
        self.engine_1_contactor.toggle(gen_1_has_output);

        self.engine_2_gen.update(engine2);
        let gen_2_has_output = self.engine_2_gen.output().is_alternating();
        self.engine_2_contactor.toggle(gen_2_has_output);

        self.apu_gen.update(apu);

        self.engine_1_contactor.powered_by(vec!(&self.engine_1_gen));
        self.ac_bus_1.powered_by(vec!(&self.engine_1_contactor));

        self.engine_2_contactor.powered_by(vec!(&self.engine_2_gen));
        self.ac_bus_2.powered_by(vec!(&self.engine_2_contactor));
    }
}

struct EngineGenerator {
    number: u8,
    output: Current,
}

impl EngineGenerator {
    const ENGINE_N2_POWER_OUTPUT_THRESHOLD: f32 = 57.5;

    fn new(number: u8) -> EngineGenerator {
        EngineGenerator {
            number,
            output: Current::None,
        }
    }

    fn update(&mut self, engine: &Engine) {
        if engine.n2 > Ratio::new::<percent>(EngineGenerator::ENGINE_N2_POWER_OUTPUT_THRESHOLD) {
            self.output = Current::Alternating(PowerSource::EngineGenerator(self.number), Frequency::new::<hertz>(400.), 
                ElectricPotential::new::<volt>(115.), ElectricCurrent::new::<ampere>(782.60));
        } else {
            self.output = Current::None
        }
    }
}

impl PowerConductor for EngineGenerator {
    fn output(&self) -> Current {
        self.output
    }
}

struct Engine {
    n2: Ratio
}

impl Engine {
    fn new() -> Engine {
        Engine {
            n2: Ratio::new::<percent>(0.)
        }
    }
}

struct ApuGenerator {
    output: Current
}

impl ApuGenerator {
    const APU_SPEED_POWER_OUTPUT_THRESHOLD: f32 = 57.5;

    fn new() -> ApuGenerator {
        ApuGenerator {
            output: Current::None
        }
    }

    fn update(&mut self, apu: &AuxiliaryPowerUnit) {
        const POWER_OUTPUT_THRESHOLD: f32 = 57.5;
        if apu.speed > Ratio::new::<percent>(ApuGenerator::APU_SPEED_POWER_OUTPUT_THRESHOLD) {
            self.output = Current::Alternating(PowerSource::ApuGenerator, Frequency::new::<hertz>(400.),
                ElectricPotential::new::<volt>(115.), ElectricCurrent::new::<ampere>(782.60));
        } else {
            self.output = Current::None
        }
    }
}

struct AuxiliaryPowerUnit {
    speed: Ratio
}

impl AuxiliaryPowerUnit {
    fn new() -> AuxiliaryPowerUnit {
        AuxiliaryPowerUnit {
            speed: Ratio::new::<percent>(0.)
        }
    }
}

struct ExternalPowerSource {
    plugged_in: bool
}

impl ExternalPowerSource {
    fn new() -> ExternalPowerSource {
        ExternalPowerSource {
            plugged_in: false
        }
    }

    fn output(&self) -> Current {
        if self.plugged_in { 
            Current::Alternating(PowerSource::External, Frequency::new::<hertz>(400.), 
                ElectricPotential::new::<volt>(115.), ElectricCurrent::new::<ampere>(782.60))
        } else {
            Current::None
        }
    }
}

struct ElectricalBus {
    input: Current
}

impl ElectricalBus {
    fn new() -> ElectricalBus {
        ElectricalBus {
            input: Current::None
        }
    }
}

impl Powerable for ElectricalBus {
    fn set_input(&mut self, current: Current) {
        self.input = current;
    }
}

impl PowerConductor for ElectricalBus {
    fn output(&self) -> Current {
        self.input
    }
}

mod tests {
    use super::*;

    fn running_engine() -> Engine {
        let mut engine = Engine::new();
        engine.n2 = Ratio::new::<percent>(EngineGenerator::ENGINE_N2_POWER_OUTPUT_THRESHOLD + 1.);

        engine
    }

    fn stopped_engine() -> Engine {
        let mut engine = Engine::new();
        engine.n2 = Ratio::new::<percent>(0.);

        engine
    }

    fn stopped_apu() -> AuxiliaryPowerUnit {
        let mut apu = AuxiliaryPowerUnit::new();
        apu.speed = Ratio::new::<percent>(0.);

        apu
    }

    #[cfg(test)]
    mod current_tests {
        use uom::si::{electric_current::ampere, electric_potential::volt, frequency::hertz};

        use super::*;

        #[test]
        fn alternating_current_is_alternating() {
            assert_eq!(alternating_current().is_alternating(), true);
        }

        #[test]
        fn alternating_current_is_not_direct() {
            assert_eq!(alternating_current().is_direct(), false);
        }

        #[test]
        fn alternating_current_is_not_none() {
            assert_eq!(alternating_current().is_none(), false);
        }

        #[test]
        fn direct_current_is_not_alternating() {
            assert_eq!(direct_current().is_alternating(), false);
        }

        #[test]
        fn direct_current_is_direct() {
            assert_eq!(direct_current().is_direct(), true);
        }

        #[test]
        fn direct_current_is_not_none() {
            assert_eq!(direct_current().is_none(), false);
        }
        
        #[test]
        fn none_current_is_not_alternating() {
            assert_eq!(none_current().is_alternating(), false);
        }

        #[test]
        fn none_current_is_not_direct() {
            assert_eq!(none_current().is_direct(), false);
        }

        #[test]
        fn none_current_is_none() {
            assert_eq!(none_current().is_none(), true);
        }

        fn alternating_current() -> Current {
            Current::Alternating(PowerSource::ApuGenerator, Frequency::new::<hertz>(0.), ElectricPotential::new::<volt>(0.), ElectricCurrent::new::<ampere>(0.))
        }

        fn direct_current() -> Current {
            Current::Direct(PowerSource::ApuGenerator, ElectricPotential::new::<volt>(0.), ElectricCurrent::new::<ampere>(0.))
        }

        fn none_current() -> Current {
            Current::None
        }
    }

    #[cfg(test)]
    mod contactor_tests {
        use super::*;
    
        #[test]
        fn contactor_starts_open() {
            assert_eq!(contactor().state, ContactorState::Open);
        }
    
        #[test]
        fn open_contactor_when_toggled_open_stays_open() {
            let mut contactor = open_contactor();
            contactor.toggle(false);
    
            assert_eq!(contactor.state, ContactorState::Open);
        }
    
        #[test]
        fn open_contactor_when_toggled_closed_closes() {
            let mut contactor = open_contactor();
            contactor.toggle(true);
    
            assert_eq!(contactor.state, ContactorState::Closed);
        }
    
        #[test]
        fn closed_contactor_when_toggled_open_opens() {
            let mut contactor = closed_contactor();
            contactor.toggle(false);
    
            assert_eq!(contactor.state, ContactorState::Open);
        }
    
        #[test]
        fn closed_contactor_when_toggled_closed_stays_closed() {
            let mut contactor = closed_contactor();
            contactor.toggle(true);
    
            assert_eq!(contactor.state, ContactorState::Closed);
        }
    
        #[test]
        fn open_contactor_has_no_output_when_powered_by_nothing() {
            contactor_has_no_output_when_powered_by_nothing(open_contactor());
        }
    
        #[test]
        fn closed_contactor_has_no_output_when_powered_by_nothing() {
            contactor_has_no_output_when_powered_by_nothing(closed_contactor());
        }
    
        fn contactor_has_no_output_when_powered_by_nothing(mut contactor: Contactor) {
            let nothing: Vec<&dyn PowerConductor> = vec![];
            contactor.powered_by(nothing);
    
            assert!(contactor.output().is_none());
        }
    
        #[test]
        fn open_contactor_has_no_output_when_powered_by_nothing_which_is_powered() {
            contactor_has_no_output_when_powered_by_nothing_which_is_powered(open_contactor());
        }
    
        #[test]
        fn closed_contactor_has_no_output_when_powered_by_nothing_which_is_powered() {
            contactor_has_no_output_when_powered_by_nothing_which_is_powered(closed_contactor());
        }
    
        fn contactor_has_no_output_when_powered_by_nothing_which_is_powered(mut contactor: Contactor) {
            contactor.powered_by(vec![&Powerless{}]);
    
            assert!(contactor.output().is_none());
        }
    
        #[test]
        fn open_contactor_has_no_output_when_powered_by_something() {
            let mut contactor = open_contactor();
            let conductors: Vec<&dyn PowerConductor> = vec![&Powerless{}, &Powered{}];
            contactor.powered_by(conductors);
    
            assert!(contactor.output().is_none());
        }
    
        #[test]
        fn closed_contactor_has_output_when_powered_by_something_which_is_powered() {
            let mut contactor = closed_contactor();
            let conductors: Vec<&dyn PowerConductor> = vec![&Powerless{}, &Powered{}];
            contactor.powered_by(conductors);
    
            assert!(contactor.output().is_alternating());
        }
    
        fn contactor() -> Contactor {
            Contactor::new()
        }
    
        fn open_contactor() -> Contactor {
            let mut contactor = contactor();
            contactor.state = ContactorState::Open;
    
            contactor
        }
    
        fn closed_contactor() -> Contactor {
            let mut contactor = contactor();
            contactor.state = ContactorState::Closed;
    
            contactor
        }
    
        struct Powerless {}
    
        impl PowerConductor for Powerless {
            fn output(&self) -> Current {
                Current::None
            }
        }
    
        struct Powered {}
    
        impl PowerConductor for Powered {
            fn output(&self) -> Current {
                Current::Alternating(PowerSource::ApuGenerator, Frequency::new::<hertz>(400.), 
                    ElectricPotential::new::<volt>(115.), ElectricCurrent::new::<ampere>(782.60))
            }
        }
    }
    
    #[cfg(test)]
    mod a320_electrical_circuit_tests {
        use super::*;
    
        #[test]
        fn starts_without_output() {
            assert!(electrical_circuit().ac_bus_1.output().is_none());
        }
    
        #[test]
        fn when_available_engine_1_gen_supplies_ac_bus_1() {
            let mut circuit = electrical_circuit();
            circuit.update(&running_engine(), &stopped_engine(), &stopped_apu());

            assert_eq!(circuit.ac_bus_1.output().get_source(), PowerSource::EngineGenerator(1));
        }

        #[test]
        fn when_available_engine_2_gen_supplies_ac_bus_2() {
            let mut circuit = electrical_circuit();
            circuit.update(&stopped_engine(), &running_engine(), &stopped_apu());

            assert_eq!(circuit.ac_bus_2.output().get_source(), PowerSource::EngineGenerator(2));
        }
    
        fn electrical_circuit() -> A320ElectricalCircuit {
            A320ElectricalCircuit::new()
        }
    }
    
    #[cfg(test)]
    mod engine_generator_tests {
        use uom::si::{ratio::percent};
        use super::*;
    
        #[test]
        fn starts_without_output() {
            assert!(engine_generator().output.is_none());
        }
    
        #[test]
        fn when_engine_n2_above_threshold_provides_output() {
            let mut generator = engine_generator();
            update_below_threshold(&mut generator);
            update_above_threshold(&mut generator);
    
            assert!(generator.output.is_alternating());
        }
    
        #[test]
        fn when_engine_n2_below_threshold_provides_no_output() {
            let mut generator = engine_generator();
            update_above_threshold(&mut generator);
            update_below_threshold(&mut generator);
    
            assert!(generator.output.is_none());
        }
    
        fn engine_generator() -> EngineGenerator {
            EngineGenerator::new(1)
        }
    
        fn engine(n2: Ratio) -> Engine {
            let mut engine = Engine::new();
            engine.n2 = n2;
    
            engine
        }
    
        fn update_above_threshold(generator: &mut EngineGenerator) {
            generator.update(&engine(Ratio::new::<percent>(EngineGenerator::ENGINE_N2_POWER_OUTPUT_THRESHOLD + 1.)));
        }
    
        fn update_below_threshold(generator: &mut EngineGenerator) {
            generator.update(&engine(Ratio::new::<percent>(EngineGenerator::ENGINE_N2_POWER_OUTPUT_THRESHOLD - 1.)));
        }
    }
    
    #[cfg(test)]
    mod apu_generator_tests {
        use uom::si::{ratio::percent};
        use super::*;
    
        #[test]
        fn starts_without_output() {
            assert!(apu_generator().output.is_none());
        }
    
        #[test]
        fn when_apu_speed_above_threshold_provides_output() {
            let mut generator = apu_generator();
            update_below_threshold(&mut generator);
            update_above_threshold(&mut generator);
    
            assert!(generator.output.is_alternating());
        }
    
        #[test]
        fn when_apu_speed_below_threshold_provides_no_output() {
            let mut generator = apu_generator();
            update_above_threshold(&mut generator);
            update_below_threshold(&mut generator);
    
            assert!(generator.output.is_none());
        }
    
        fn apu_generator() -> ApuGenerator {
            ApuGenerator::new()
        }
    
        fn apu(speed: Ratio) -> AuxiliaryPowerUnit {
            let mut apu = AuxiliaryPowerUnit::new();
            apu.speed = speed;
    
            apu
        }
    
        fn update_above_threshold(generator: &mut ApuGenerator) {
            generator.update(&apu(Ratio::new::<percent>(ApuGenerator::APU_SPEED_POWER_OUTPUT_THRESHOLD + 1.)));
        }
    
        fn update_below_threshold(generator: &mut ApuGenerator) {
            generator.update(&apu(Ratio::new::<percent>(ApuGenerator::APU_SPEED_POWER_OUTPUT_THRESHOLD - 1.)));
        }
    }
    
    #[cfg(test)]
    mod external_power_source_tests {
        use super::*;
    
        #[test]
        fn starts_without_output() {
            assert!(external_power_source().output().is_none());
        }
    
        #[test]
        fn when_plugged_in_provides_output() {
            let mut ext_pwr = external_power_source();
            ext_pwr.plugged_in = true;
    
            assert!(ext_pwr.output().is_alternating());
        }
    
        #[test]
        fn when_not_plugged_in_provides_no_output() {
            let mut ext_pwr = external_power_source();
            ext_pwr.plugged_in = false;
    
            assert!(ext_pwr.output().is_none());
        }
    
        fn external_power_source() -> ExternalPowerSource {
            ExternalPowerSource::new()
        }
    }
    
}