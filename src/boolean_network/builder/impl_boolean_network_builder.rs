use crate::boolean_network::builder::{BooleanNetworkBuilder, RegulationTemplate, UpdateFunctionTemplate, RegulatoryGraph};

impl BooleanNetworkBuilder {

    pub fn new_from_templates(regulations: Vec<RegulationTemplate>, update_functions: Vec<UpdateFunctionTemplate>) -> Result<BooleanNetworkBuilder, String> {
        let rg = RegulatoryGraph::from_regulations(regulations);

        // first, replace variables for unary parameters based on known variables
        let update_functions: Vec<UpdateFunctionTemplate> = update_functions.into_iter().map(|f| {
            *f.swap_unary_parameters(&rg)
        }).collect();



        return Err("foo".to_string())
    }

}