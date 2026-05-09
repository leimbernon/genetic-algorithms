mod structures;
mod engines {
    mod test_ga;
    mod test_examples;
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
        mod test_pareto;
    }
    mod nsga3 {
        mod test_das_dennis;
        mod test_nsga3;
        mod test_nsga3_configuration;
    }
}
