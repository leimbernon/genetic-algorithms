mod structures;
mod engines {
    mod test_ga;
    mod test_examples;
    mod local_search;
    mod alps {
        mod test_alps;
    }
    mod cellular {
        mod test_cellular;
    }
    mod de {
        mod test_de;
    }
    mod scatter {
        mod test_scatter;
    }
    mod cma {
        mod test_cma;
    }
    mod island {
        mod test_island;
        mod test_island_configuration;
        mod test_island_migration;
        mod test_island_nsga2;
        mod test_island_topology;
    }
    mod nsga2 {
        mod test_crowding_distance;
        mod test_non_dominated_sort;
        mod test_nsga2;
        mod test_nsga2_configuration;
        mod test_nsga2_constraints;
        mod test_pareto;
    }
    mod nsga3 {
        mod test_das_dennis;
        mod test_nsga3;
        mod test_nsga3_configuration;
    }
    mod moead {
        mod test_moead;
        mod test_moead_configuration;
    }
    mod spea2 {
        mod test_spea2;
        mod test_spea2_configuration;
    }
    mod sms_emoa {
        mod test_sms_emoa;
        mod test_sms_emoa_configuration;
    }
    mod ibea {
        mod test_ibea;
        mod test_ibea_configuration;
    }
    mod hall_of_fame {
        mod test_hall_of_fame;
    }
    mod warm_starting {
        mod test_warm_starting;
    }
    mod aos {
        mod test_aos;
    }
    mod test_strategy_trait;
    mod hill_climb {
        mod test_hill_climb;
    }
    mod permutate {
        mod test_permutate;
    }
}
