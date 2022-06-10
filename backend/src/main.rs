use std::sync;
use std::sync::atomic;
use std::thread;
use tauri;

mod app;
mod daw;
mod util;

#[tauri::command]
fn toggle_playlist(state: tauri::State<'_, sync::Arc<daw::InnerState>>) {
  if state.playlist_is_playing.load(atomic::Ordering::SeqCst) {
    daw::pause_playlist(state);
  } else {
    // start playlist
    daw::start_playlist(state);
  }
}

#[tauri::command]
fn get_playlist_playing(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>
) -> Result<bool, String> {
  Ok(state.playlist_is_playing.load(atomic::Ordering::SeqCst))
}

#[tauri::command]
fn get_playlist_start_time(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>
) -> Result<i64, String> {
  Ok(state.playlist_started_time.load(atomic::Ordering::SeqCst))
}

#[tauri::command]
fn get_playlist_tempo(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>
) -> Result<f32, String> {
  Ok(*state.global_tempo_bpm.lock().unwrap())
}

#[tauri::command]
fn set_playlist_tempo(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>,
  val: f32,
) {
  println!("playlist tempo updated: {}", val);
  daw::set_playlist_tempo(state, val);
}

#[tauri::command]
fn toggle_metronome_enabled(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>
) {
  let val = !state.metronome_enabled.load(atomic::Ordering::SeqCst);
  state.metronome_enabled.store(val, atomic::Ordering::SeqCst);
}

#[tauri::command]
fn get_metronome_enabled(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>
) -> Result<bool, String> {
  Ok(state.metronome_enabled.load(atomic::Ordering::SeqCst))
}

#[tauri::command]
fn get_playlist_runtime_formatted(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>
) -> Result<String, String> {
  let res = util::format_playlist_runtime(
    state.playlist_started_time.load(atomic::Ordering::SeqCst),
  );
  Ok(res)
}

#[tauri::command]
fn get_playlist_time_signature(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>
) -> Result<(u16, u16), String> {
  let res = state.playlist_time_signature.lock().unwrap();
  Ok((res.numerator, res.denominator))
}

#[tauri::command]
fn set_playlist_time_signature(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>,
  numerator: u16,
  denominator: u16,
) {
  let updated: daw::timing::TimeSignature = daw::timing::TimeSignature {
    numerator,
    denominator,
  };
  *state.playlist_time_signature.lock().unwrap() = updated;
}

#[tauri::command]
fn get_sidebar_samples() -> Result<Vec<String>, String> {
  Ok(app::get_sidebar_samples())
}

#[tauri::command]
fn preview_sample(
  _state: tauri::State<'_, sync::Arc<daw::InnerState>>,
  path: String,
) {
  thread::spawn(move || {
    futures::executor::block_on(daw::play_sample(&path));
  });
}

#[tauri::command]
fn get_audio_drivers() -> Result<Vec<String>, String> {
  Ok(daw::drivers::get_sound_host_names())
}

#[tauri::command]
fn add_audiograph_node(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>,
  sample_path: String,
  start_offset: u64,
  track_number: u32,
) -> Result<u64, String> {
  let id = state
    .playlist_audiograph
    .lock()
    .unwrap()
    .construct_and_add_node(sample_path, start_offset, track_number);

  // returns the id of the new node
  Ok(id)
}

#[tauri::command]
fn move_audiograph_node(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>, 
  id: u64,
  start_offset: u64,
  track_number: u32,
) {
  // set new start offset
  state
    .playlist_audiograph
    .lock()
    .unwrap()
    .get_mut_node(id)
    .start_offset = start_offset;
  
  // set new track number
  state
    .playlist_audiograph
    .lock()
    .unwrap()
    .get_mut_node(id)
    .track_number = track_number;

  // manually resort nodes by sort time
  // after moving a node because the user might
  // have changed the order of nodes in the playlist,
  // normally a sort happens when a new node is added
  state
    .playlist_audiograph
    .lock()
    .unwrap()
    .nodes
    .sort_unstable_by(|a, b| a.start_offset.cmp(&b.start_offset));
}

#[tauri::command]
fn remove_audiograph_node(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>, 
  id: u64,
) {
  state
    .playlist_audiograph
    .lock()
    .unwrap()
    .remove_node(id);
}

#[tauri::command]
fn get_node_data(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>,
  id: u64,
) -> Result<(String, String), String> {
  let (svg_pathd, svg_viewbox) = state
    .playlist_audiograph
    .lock()
    .unwrap()
    .nodes
    .iter()
    .find(|&el| { el.id == id })
    .unwrap()
    .get_waveform();

  // return the svg path
  // todo:
  // in the future this method will 
  // return other data on nodes for
  // populating the timeline graph
  Ok((svg_pathd, svg_viewbox))
}

#[tauri::command]
fn get_playlist_sample_offset(
  state: tauri::State<'_, sync::Arc<daw::InnerState>>,
  drop_x: f32,
  drop_y: f32,
  min_bound_x: f32,
  min_bound_y: f32,
  max_bound_x: f32,
  max_bound_y: f32,
) -> Result<u64, String> {
  // todo: choose a number that isn't arbitrary
  let max_sample_offset = (max_bound_x - min_bound_x).round() as u64 * 5;
  println!("max sample offset: {}", max_sample_offset);
  let res = util::calc_playlist_sample_offset(
    drop_x,
    drop_y,
    min_bound_x,
    min_bound_y,
    max_bound_x,
    max_bound_y,
    max_sample_offset,
  );

  Ok(res)
}

fn main() {
  tauri::Builder::default()
    .setup(app::setup)
    .menu(app::build_menu())
    .manage(sync::Arc::new(daw::InnerState::default()))
    .invoke_handler(tauri::generate_handler![
      get_playlist_playing,
      toggle_playlist,
      get_playlist_start_time,
      get_playlist_tempo,
      set_playlist_tempo,
      toggle_metronome_enabled,
      get_metronome_enabled,
      get_playlist_runtime_formatted,
      get_playlist_time_signature,
      set_playlist_time_signature,
      get_sidebar_samples,
      preview_sample,
      get_audio_drivers,
      add_audiograph_node,
      move_audiograph_node,
      remove_audiograph_node,
      get_node_data,
      get_playlist_sample_offset
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
