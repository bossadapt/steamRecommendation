use charts::{Chart, ScaleBand, ScaleLinear, VerticalBarView};
use chrono::{format::format, NaiveDate};
use plotters::prelude::*;
use serde::Deserialize;
use std::{
    array,
    collections::{hash_set, HashMap, HashSet},
    fs::File,
    io::{self, BufRead},
    path,
};
#[derive(Debug, serde::Deserialize)]
struct Recommendation {
    app_id: usize,
    helpful: usize,
    funny: usize,
    date: NaiveDate,
    is_recommended: bool,
    hours: f32,
    user_id: usize,
    review_id: usize,
}
#[derive(Debug, serde::Deserialize, Clone)]
struct Game {
    app_id: usize,
    title: String,
    date_release: NaiveDate,
    win: bool,
    mac: bool,
    linux: bool,
    rating: String,
    positive_ratio: u8,
    user_reviews: usize,
    price_final: f32,
    price_original: f32,
    discount: f32,
    steam_deck: bool,
}
#[derive(Debug, serde::Deserialize)]
struct User {
    user_id: usize,
    products: usize,
    reviews: usize,
}
#[derive(Debug, serde::Deserialize)]
struct GamesMetadata {
    app_id: usize,
    description: String,
    tags: Vec<String>,
}
fn main() {
    //converting files to structures
    let mut recommendations: Vec<Recommendation> = csv_to_vector("data\\recommendations.csv");
    let games: Vec<Game> = csv_to_vector("data\\games.csv");
    //let users: Vec<User> = csv_to_vector("data\\users.csv");
    let games_metadata: Vec<GamesMetadata> = jsonl_to_vector("data\\games_metadata.json");

    //getting expected ratios
    let expected_positive_ratio = {
        let mut total: f32 = 0.0;
        for game in &games {
            total += game.positive_ratio as f32;
        }
        total / (games.len() as f32)
    };
    let expected_recommend_ratio = {
        let mut total: f64 = 0.0;
        for recomendation in &recommendations {
            if recomendation.is_recommended {
                total += 1.0;
            }
        }
        total / (recommendations.len() as f64)
    };

    //sorting the recomendations decending
    recommendations.sort_unstable_by(|a, b| a.hours.partial_cmp(&b.hours).unwrap());
    let games: HashMap<_, _> = games.into_iter().map(|game| (game.app_id, game)).collect();

    //drawing the drawables with data
    draw_tag_chart(games.clone(), &games_metadata, 10, 10, 500);
    draw_tag_chart(games.clone(), &games_metadata, 10, 10, 1000);
    draw_tag_chart(games.clone(), &games_metadata, 10, 10, 1500);
    println!();
    ////////////////////////////////////////////////////////////////
    draw_description_word_chart(games.clone(), &games_metadata, 10, 10, 500);
    draw_description_word_chart(games.clone(), &games_metadata, 10, 10, 1000);
    draw_description_word_chart(games.clone(), &games_metadata, 10, 10, 1500);
    println!("Expected positive ratio: {}", expected_positive_ratio);
    println!();
    println!();
    ////////////////////////////////////////////////////////////////
    draw_recommendation_hours_graph(&recommendations, "recomendationxhours".to_owned());
    println!("Expected recommend ratio: {}", expected_recommend_ratio);
    println!("Finished Running")
}
//c
fn draw_recommendation_hours_graph(recommendations: &[Recommendation], file_name: String) {
    let mut rec_list: Vec<(f32, f32)> = Vec::new();
    for i in 0..=1000 {
        let current_ratio = (get_recommend_ratio_based_on_hours(
            &recommendations,
            i as f64,
            (i + 1) as f64,
        ) * 100.0) as f32;
        if current_ratio != 0.0 {
            rec_list.push((i as f32, current_ratio));
            println!("{}:hours ({},{}) = {}", i, i, i + 1, current_ratio)
        } else {
            println!("excluded hour: {}:{}", i, current_ratio)
        }
    }
    draw_graph_given_points(
        "Steam Recommendation Average By Hours Played".to_owned(),
        rec_list,
        file_name,
    )
    .expect("failed to draw graph");
}
fn draw_tag_chart(
    games: HashMap<usize, Game>,
    games_metadata: &[GamesMetadata],
    top_count: usize,
    bottom_count: usize,
    min_appearance: usize,
) {
    let file_name = format!("tag{}x{}x{}", top_count, bottom_count, min_appearance);
    let mut tag_average_rating = get_tag_average_rating(games, games_metadata, min_appearance);
    tag_average_rating.sort_unstable_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
    let mut tag_chart_data: Vec<(String, f32)> = Vec::new();
    for i in 0..top_count {
        let (tag, avg_rating) = &tag_average_rating[i];
        tag_chart_data.push((tag.clone(), avg_rating.clone()));
        println!(
            "Including rank(tag) {}: {}({})",
            tag_average_rating.len() - i + 1,
            tag,
            avg_rating
        );
    }
    let second_index = 0..bottom_count;
    for i in second_index.rev() {
        let (tag, avg_rating) = &tag_average_rating[tag_average_rating.len() - i - 1];
        tag_chart_data.push((tag.clone(), avg_rating.clone()));
        println!("Including(tag) rank {}: {}({})", i + 1, tag, avg_rating);
    }
    let file_path = format!(".\\output\\${}.svg", file_name);
    let title: String = format!("Tag rating (min appearance: {})", min_appearance);
    draw_chart_given_data(
        &tag_chart_data,
        title,
        "Tag".to_owned(),
        "Average Rating".to_owned(),
        file_path,
    );
}
fn draw_description_word_chart(
    games: HashMap<usize, Game>,
    games_metadata: &[GamesMetadata],
    top_count: usize,
    bottom_count: usize,
    min_appearance: usize,
) {
    //top count for changing the index of whats being added to the bar chart
    let file_name = format!("desc{}x{}x{}", top_count, bottom_count, min_appearance);
    let mut word_average_ratings =
        get_description_word_average_rating(&games, &games_metadata, min_appearance);
    word_average_ratings.sort_unstable_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
    let mut word_chart_data: Vec<(String, f32)> = Vec::new();
    for index in 0..bottom_count {
        let (word, average_ratio) = word_average_ratings.get(index).unwrap().to_owned();
        word_chart_data.push((word.clone(), average_ratio.clone()));
        println!(
            "Including rank(desc) {}: {}({})",
            word_average_ratings.len() - index + 1,
            word,
            average_ratio
        );
    }
    let second_range = 0..top_count;
    for index2 in second_range.rev() {
        let (word, average_ratio) = word_average_ratings
            .get(word_average_ratings.len() - index2 - 1)
            .unwrap()
            .to_owned();
        word_chart_data.push((word.clone(), average_ratio.clone()));
        println!(
            "Including rank(desc) {}: {}({})",
            index2 + 1,
            word,
            average_ratio
        );
    }
    let file_path = format!(".\\output\\${}.svg", file_name);
    let title: String = format!("Description rating (min appearance: {})", min_appearance);
    draw_chart_given_data(
        &word_chart_data,
        title,
        "Word".to_owned(),
        "Average Rating".to_owned(),
        file_path,
    );
}
fn draw_chart_given_data(
    data: &[(String, f32)],
    bar_chart_title: String,
    x_title: String,
    y_title: String,
    output_file_path: String,
) {
    let width = 1920;
    let height = 1080;
    let (top, right, bottom, left) = (90, 0, 50, 60);

    let x_row = data
        .into_iter()
        .map(|(title, _average)| title.to_owned())
        .collect::<Vec<String>>();

    let x = ScaleBand::new()
        .set_domain(x_row)
        .set_range(vec![0, width - left - right])
        .set_inner_padding(0.2)
        .set_outer_padding(0.2);

    let y = ScaleLinear::new()
        .set_domain(vec![0.0, 100.0])
        .set_range(vec![height - top - bottom, 0]);

    let view = VerticalBarView::new()
        .set_x_scale(&x)
        .set_y_scale(&y)
        .load_data(&data.to_vec())
        .unwrap();
    // Generate and save the chart.
    Chart::new()
        .set_width(width)
        .set_height(height)
        .set_margins(top, right, bottom, left)
        .add_title(String::from(bar_chart_title))
        .add_view(&view)
        .add_axis_bottom(&x)
        .add_axis_left(&y)
        .add_left_axis_label(y_title)
        .add_bottom_axis_label(x_title)
        .save(output_file_path.to_owned())
        .unwrap();
    println!("chart created at {}", output_file_path);
}
fn draw_graph_given_points(
    graph_title: String,
    points_tuple_vec: Vec<(f32, f32)>,
    file_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_output = format!(".\\output\\{}.png", file_name);
    let root = BitMapBackend::new(&file_output, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;
    let root = root.margin(10, 10, 10, 10);
    // After this point, we should be able to construct a chart context
    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption(graph_title, ("sans-serif", 40).into_font())
        // Set the size of the label region
        .x_label_area_size(40)
        .y_label_area_size(80)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(0f32..1000f32, 0f32..100f32)?;

    // Then we can draw a mesh
    // make sure
    chart
        .configure_mesh()
        .x_desc("Game Time by Hours")
        .y_desc("Positive Ratio Average")
        .axis_desc_style(("sans-serif", 20))
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(100)
        .y_labels(10)
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw()?;
    chart.draw_series(LineSeries::new(points_tuple_vec, &RED))?;
    root.present()?;
    println!("graph created at {}", file_output);
    Ok(())
}
//https://github.com/plotters-rs/plotters
fn get_recommend_ratio_based_on_hours(
    recommendations: &[Recommendation],
    min_hours: f64,
    max_hours: f64,
) -> f64 {
    let mut total: f64 = 0.0;
    let mut rec_total: f64 = 0.0;
    let current_range = min_hours..=max_hours;
    let start_index: usize = match recommendations
        .binary_search_by(|r| r.hours.partial_cmp(&(min_hours as f32)).unwrap())
    {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    };
    let mut cur_index = start_index;
    while cur_index < recommendations.len()
        && current_range.contains(&(recommendations[cur_index].hours as f64))
    {
        if recommendations[cur_index].is_recommended {
            rec_total += 1.0;
            total += 1.0;
        } else {
            total += 1.0;
        }
        cur_index += 1;
    }
    let outcome: f64 = {
        if total == 0.0 {
            0.0
        } else {
            rec_total / total
        }
    };
    return outcome;
}

fn get_tag_average_rating(
    games: HashMap<usize, Game>,
    games_metadata: &[GamesMetadata],
    min_appearance: usize,
) -> Vec<(String, f32)> {
    let mut tag_ratings: HashMap<String, Vec<u8>> = HashMap::new();
    for game in games_metadata {
        let current_game = games.get(&game.app_id).unwrap();
        let current_tags = &game.tags;
        for tag in current_tags {
            tag_ratings
                .entry(tag.to_lowercase().clone())
                .and_modify(|ratings| ratings.push(current_game.positive_ratio))
                .or_insert_with(|| vec![current_game.positive_ratio]);
        }
    }
    let mut tag_ratings_averaged: Vec<(String, f32)> = Vec::new();
    for (word, scores) in tag_ratings {
        if scores.len() >= min_appearance {
            let average: f32 = {
                let total: f32 = scores.iter().map(|&x| x as f32).sum();
                total / (scores.len() as f32)
            };
            tag_ratings_averaged.push((word, average));
        }
    }
    return tag_ratings_averaged;
}
fn get_description_word_average_rating(
    games: &HashMap<usize, Game>,
    games_metadata: &[GamesMetadata],
    min_appearance: usize,
) -> Vec<(String, f32)> {
    let mut word_ratings: HashMap<String, Vec<u8>> = HashMap::new();
    for game in games_metadata {
        let current_game = games.get(&game.app_id).unwrap();
        let description = game.description.to_lowercase();
        let description: String = description
            .chars()
            .filter(|x| match x {
                ' ' => return true,
                'a'..='z' => return true,
                _ => return false,
            })
            .collect();
        //convert to a string hashset excluding special characters
        let description_list = description.split(' ');
        let mut new_words: HashSet<String> = HashSet::new();
        for word in description_list {
            new_words.insert(word.to_owned());
        }
        for word in new_words {
            word_ratings
                .entry(word)
                .and_modify(|ratings| ratings.push(current_game.positive_ratio))
                .or_insert_with(|| vec![current_game.positive_ratio]);
        }
    }
    let mut word_ratings_averaged: Vec<(String, f32)> = Vec::new();
    for (word, scores) in word_ratings {
        if scores.len() >= min_appearance {
            let average: f32 = {
                let total: f32 = scores.iter().map(|&x| x as f32).sum();
                total / (scores.len() as f32)
            };
            word_ratings_averaged.push((word, average));
        }
    }
    return word_ratings_averaged;
}
fn jsonl_to_vector<T: for<'de> Deserialize<'de>>(json_path: &str) -> Vec<T> {
    let file = File::open(json_path).unwrap();
    let mut games_metadata: Vec<T> = Vec::new();
    for line in io::BufReader::new(file).lines() {
        games_metadata.push(serde_json::from_str(&line.unwrap()).unwrap());
    }
    return games_metadata;
}
fn csv_to_vector<T: for<'de> Deserialize<'de>>(csv_path: &str) -> Vec<T> {
    let mut vector_to_return: Vec<T> = Vec::new();
    let file = File::open(csv_path).unwrap();
    let mut rdr = csv::Reader::from_reader(io::BufReader::new(file));
    for result in rdr.deserialize() {
        let record: T = result.unwrap();
        vector_to_return.push(record);
    }
    return vector_to_return;
}
