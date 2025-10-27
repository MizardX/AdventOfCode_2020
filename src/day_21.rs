use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Allergen {
    Dairy,
    Eggs,
    Fish,
    Nuts,
    Peanuts,
    Sesame,
    Shellfish,
    Soy,
    Wheat,
}

impl Allergen {
    const fn all() -> [Self; 9] {
        [
            Self::Dairy,
            Self::Eggs,
            Self::Fish,
            Self::Nuts,
            Self::Peanuts,
            Self::Sesame,
            Self::Shellfish,
            Self::Soy,
            Self::Wheat,
        ]
    }
}

impl FromStr for Allergen {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "dairy" => Self::Dairy,
            "eggs" => Self::Eggs,
            "fish" => Self::Fish,
            "nuts" => Self::Nuts,
            "peanuts" => Self::Peanuts,
            "sesame" => Self::Sesame,
            "shellfish" => Self::Shellfish,
            "soy" => Self::Soy,
            "wheat" => Self::Wheat,
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Food {
    ingredients: Vec<usize>,
    allergens: Vec<Allergen>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FoodList {
    lookup: HashMap<Rc<str>, usize>,
    names: Vec<Rc<str>>,
    foods: Vec<Food>,
}

impl FromStr for FoodList {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lookup = HashMap::new();
        let mut names = Vec::new();
        let mut foods = Vec::new();
        for line in s.lines() {
            let (ingredients, rest) = line
                .split_once(" (contains ")
                .ok_or(ParseError::SyntaxError)?;
            let ingredients = ingredients
                .split(' ')
                .map(|name| {
                    if let Some(&index) = lookup.get(name) {
                        index
                    } else {
                        let index = names.len();
                        let rc: Rc<str> = Rc::from(name);
                        names.push(rc.clone());
                        lookup.insert(rc, index);
                        index
                    }
                })
                .collect::<Vec<_>>();
            let allergens = rest
                .strip_suffix(")")
                .ok_or(ParseError::SyntaxError)?
                .split(", ")
                .map(str::parse)
                .collect::<Result<Vec<_>, _>>()?;
            foods.push(Food {
                ingredients,
                allergens,
            });
        }
        Ok(Self {
            lookup,
            names,
            foods,
        })
    }
}

#[aoc_generator(day21)]
fn parse(input: &str) -> Result<FoodList, ParseError> {
    input.parse()
}

#[aoc(day21, part1)]
fn part_1(food_list: &FoodList) -> usize {
    let ingredient_per_allergen = determine_allergenic_ingredients(food_list);
    let mut non_allergen_count = 0;
    for food in &food_list.foods {
        non_allergen_count += food
            .ingredients
            .iter()
            .filter(|&i| !ingredient_per_allergen.contains(i))
            .count();
    }
    non_allergen_count
}

#[aoc(day21, part2)]
fn part_2(food_list: &FoodList) -> String {
    let ingredient_per_allergen = determine_allergenic_ingredients(food_list);
    let mut result = String::new();
    for ingredient in ingredient_per_allergen {
        let ingredient = &food_list.names[ingredient];
        if !result.is_empty() {
            result.push(',');
        }
        result.push_str(ingredient);
    }
    result
}

fn determine_allergenic_ingredients(food_list: &FoodList) -> Vec<usize> {
    let ingredients_per_allergen = Allergen::all()
        .iter()
        .map(|allergen| {
            let mut possible_ingredients: HashSet<usize> = (0..food_list.names.len()).collect();
            for food in &food_list.foods {
                if food.allergens.contains(allergen) {
                    possible_ingredients.retain(|i| food.ingredients.contains(i));
                }
            }
            possible_ingredients
        })
        .collect::<Vec<_>>();
    let mut locked_ingredient_per_allergen = Allergen::all().map(|_| None);
    let mut ingredients_with_allergens = HashSet::new();
    while let Some(candidate) = Allergen::all().into_iter().find(|&a| {
        locked_ingredient_per_allergen[a as usize].is_none()
            && ingredients_per_allergen[a as usize]
                .iter()
                .filter(|&&i| !ingredients_with_allergens.contains(&i))
                .count()
                == 1
    }) {
        let ingredient = ingredients_per_allergen[candidate as usize]
            .iter()
            .find_map(|&i| (!ingredients_with_allergens.contains(&i)).then_some(i))
            .unwrap();
        ingredients_with_allergens.insert(ingredient);
        locked_ingredient_per_allergen[candidate as usize] = Some(ingredient);
    }
    locked_ingredient_per_allergen
        .into_iter()
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        mxmxvkd kfcds sqjhc nhms (contains dairy, fish)\n\
        trh fvjkl sbzzf mxmxvkd (contains dairy)\n\
        sqjhc fvjkl (contains soy)\n\
        sqjhc mxmxvkd sbzzf (contains fish)\
    ";

    #[test]
    fn test_parse() {
        let food_list = parse(EXAMPLE).unwrap();
        assert_eq!(
            food_list.foods,
            [
                Food {
                    ingredients: vec![
                        food_list.lookup["mxmxvkd"],
                        food_list.lookup["kfcds"],
                        food_list.lookup["sqjhc"],
                        food_list.lookup["nhms"]
                    ],
                    allergens: vec![Allergen::Dairy, Allergen::Fish]
                },
                Food {
                    ingredients: vec![
                        food_list.lookup["trh"],
                        food_list.lookup["fvjkl"],
                        food_list.lookup["sbzzf"],
                        food_list.lookup["mxmxvkd"]
                    ],
                    allergens: vec![Allergen::Dairy]
                },
                Food {
                    ingredients: vec![food_list.lookup["sqjhc"], food_list.lookup["fvjkl"]],
                    allergens: vec![Allergen::Soy]
                },
                Food {
                    ingredients: vec![
                        food_list.lookup["sqjhc"],
                        food_list.lookup["mxmxvkd"],
                        food_list.lookup["sbzzf"]
                    ],
                    allergens: vec![Allergen::Fish]
                }
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let food_lits = parse(EXAMPLE).unwrap();
        let result = part_1(&food_lits);
        assert_eq!(result, 5);
    }
    
    #[test]
    fn test_part_2() {
        let food_lits = parse(EXAMPLE).unwrap();
        let result = part_2(&food_lits);
        assert_eq!(result, "mxmxvkd,sqjhc,fvjkl");
    }
}
