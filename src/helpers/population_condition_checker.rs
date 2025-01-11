use crate::configuration::GaConfiguration;

/**
 * Function to check that the population size is set
 */
pub fn check_population_size_is_set(configuration: &GaConfiguration){
    if configuration.limit_configuration.population_size <= 0 {
        panic!("The population size must be set.");
    }
}