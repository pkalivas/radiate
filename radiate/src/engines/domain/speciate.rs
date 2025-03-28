use super::random_provider;
use crate::{Chromosome, Phenotype, Population, Score, Species, objectives};
use std::collections::HashMap;

pub fn generate_mascots<C: Chromosome>(population: &Population<C>, species: &mut Vec<Species<C>>) {
    let mut to_remove = Vec::new();
    for (idx, species) in species.iter_mut().enumerate() {
        let species_members = population
            .iter()
            .filter(|phenotype| phenotype.species_id() == Some(species.id()))
            .collect::<Vec<&Phenotype<C>>>();

        if species_members.is_empty() {
            to_remove.push(idx);
            continue;
        }

        let random_phenotype = random_provider::choose(&species_members);
        species.set_mascot(random_phenotype.genotype().clone());
        species.set_count(0);
    }

    for idx in to_remove.iter().rev() {
        species.remove(*idx);
    }
}

pub fn fitness_share<C: Chromosome>(
    population: &mut Population<C>,
    species: &mut Vec<Species<C>>,
    objective: &objectives::Objective,
) {
    let mut species_members = population
        .iter_mut()
        .filter(|phenotype| phenotype.species_id().is_some())
        .fold(HashMap::new(), |mut map, phenotype| {
            let species_id = phenotype.species_id().unwrap();
            map.entry(species_id)
                .or_insert_with(Vec::new)
                .push(phenotype);
            map
        });

    species_members.retain(|_, members| !members.is_empty());
    species.retain(|species| species_members.contains_key(&species.id()));

    let mut total_species_score = Score::default();
    for i in 0..species.len() {
        let members = scaled_member_scores(&species_members[&species[i].id()]);
        total_species_score = total_species_score + members.iter().sum::<Score>();
    }

    for i in 0..species.len() {
        let species = &mut species[i];
        let members = species_members.get_mut(&species.id()).unwrap();

        let adjusted_score =
            scaled_member_scores(&members).iter().sum::<Score>() / total_species_score.clone();

        let mut best_score = members
            .iter()
            .filter_map(|member| member.score())
            .collect::<Vec<Score>>();

        best_score.sort_by(|a, b| objective.cmp(a, b));

        species.set_count(members.len());
        species.update_score(adjusted_score, best_score[0].clone(), objective);
        for member in members.iter_mut() {
            member.set_species_id(Some(species.id()));
        }
    }

    species.sort_by(|a, b| objective.cmp(a.score(), b.score()));

    let scores = species
        .iter()
        .map(|species| species.score().clone())
        .collect::<Vec<Score>>();

    let mut idx = species.len();
    for i in 0..scores.len() {
        let species = &mut species[i];
        species.set_score(scores[idx - 1].clone());
        idx -= 1;
    }
}

fn scaled_member_scores<C: Chromosome>(members: &[&mut Phenotype<C>]) -> Vec<Score> {
    members
        .iter()
        .map(|member| member.score().unwrap().clone() / members.len() as f32)
        .collect::<Vec<Score>>()
}
