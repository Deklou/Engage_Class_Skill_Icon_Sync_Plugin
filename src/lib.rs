use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use unity::il2cpp;
use unity::prelude::*;
use unity::system::{Il2CppString, List};

#[unity::class("App", "SkillData")]
struct SkillData {}
#[unity::class("UnityEngine", "Sprite")]
struct Sprite {}
#[unity::class("UnityEngine", "Texture")]
struct Texture {}
#[unity::class("UnityEngine", "Material")]
struct Material {}
#[unity::class("TMPro", "TMP_SpriteAsset")]
struct SpriteAsset {}
#[unity::class("TMPro", "TMP_SpriteGlyph")]
struct SpriteGlyph {}
#[unity::class("TMPro", "TMP_SpriteCharacter")]
struct SpriteCharacter {}

// Offsets et structures du jeu 2.0.0 (Ghidra main_200).
const PREFIX: &str = "ClassSkillIconSync_";
static NEXT_ID: AtomicU32 = AtomicU32::new(1);
static ASSETS: OnceLock<Mutex<HashMap<String, (String, usize, u32)>>> = OnceLock::new();
fn assets() -> &'static Mutex<HashMap<String, (String, usize, u32)>> {
    ASSETS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[skyline::from_offset(0x024899B0)]
fn get_icon_label(skill: *const SkillData, method: OptionalMethod) -> *const Il2CppString;
#[skyline::from_offset(0x0227D110)]
fn get_skill_icon(label: *const Il2CppString, method: OptionalMethod) -> *mut Sprite;
#[skyline::from_offset(0x02F97670)]
fn sprite_texture(sprite: *const Sprite, method: OptionalMethod) -> *mut Texture;
#[skyline::from_offset(0x037894F0)]
fn texture_width(texture: *const Texture, method: OptionalMethod) -> i32;
#[skyline::from_offset(0x03789580)]
fn texture_height(texture: *const Texture, method: OptionalMethod) -> i32;
#[skyline::from_offset(0x02F8C730)]
fn create_scriptable_object(ty: *const unity::system::SystemType, method: OptionalMethod) -> *mut SpriteAsset;
#[skyline::from_offset(0x032D9160)]
fn material_ctor(this: *mut Material, original: *const Material, method: OptionalMethod);
#[skyline::from_offset(0x032D9950)]
fn set_material_texture(this: *mut Material, texture: *const Texture, method: OptionalMethod);
#[skyline::from_offset(0x032EE5F0)]
fn set_object_name(this: *mut SpriteAsset, name: *const Il2CppString, method: OptionalMethod);
#[skyline::from_offset(0x0282D7C0)]
fn sprite_glyph_ctor(this: *mut SpriteGlyph, method: OptionalMethod);
#[skyline::from_offset(0x0282DA20)]
fn sprite_character_ctor(this: *mut SpriteCharacter, unicode: u32, asset: *mut SpriteAsset, glyph: *mut SpriteGlyph, method: OptionalMethod);
#[skyline::from_offset(0x0282D800)]
fn set_character_name(this: *mut SpriteCharacter, name: *const Il2CppString, method: OptionalMethod);
#[skyline::from_offset(0x0282B950)]
fn update_lookup_tables(asset: *mut SpriteAsset, method: OptionalMethod);
#[skyline::from_offset(0x025D5020)]
fn original_asset_callback(arg: i32, asset_name: *const Il2CppString, method: OptionalMethod) -> *mut SpriteAsset;

#[skyline::from_offset(0x02D8DAE0)]
fn register_sprite_asset(hash: i32, asset: *mut SpriteAsset, method: OptionalMethod);

// Les offsets de champs incluent l'en-tête IL2CPP de 0x10 octets.
unsafe fn field<T: Copy>(ptr: *const u8, offset: usize) -> T {
    (ptr.add(offset) as *const T).read()
}
unsafe fn set_field<T>(ptr: *mut u8, offset: usize, value: T) {
    (ptr.add(offset) as *mut T).write(value);
}

unsafe fn build_asset(name: &str, label: &Il2CppString, sprite: *mut Sprite, texture: *mut Texture, id: u32)
    -> Option<(*mut SpriteAsset, u32)> {
    let stock = original_asset_callback(0, Il2CppString::new("Skill"), None);
    if stock.is_null() { return None; }
    let stock_material: *mut Material = field(stock.cast(), 0x20);
    if stock_material.is_null() { return None; }

    let class = il2cpp::class::Il2CppClass::from_name("TMPro", "TMP_SpriteAsset").ok()?;
    let asset = create_scriptable_object(class.get_system_type(), None);
    if asset.is_null() { return None; }
    let material = il2cpp::instantiate_class_by_name::<Material>("UnityEngine", "Material").ok()?;
    material_ctor(material, stock_material, None);
    set_material_texture(material, texture, None);
    set_object_name(asset, Il2CppString::new(name), None);
    set_field(asset.cast(), 0x20, material as *mut Material);
    set_field(asset.cast(), 0xA8, texture);

    // Réutilise la version et les métriques TMP d'origine.
    let version: *const Il2CppString = field(stock.cast(), 0x40);
    set_field(asset.cast(), 0x40, version);
    std::ptr::copy_nonoverlapping(stock.cast::<u8>().add(0x48), asset.cast::<u8>().add(0x48), 96);

    let glyph = il2cpp::instantiate_class_by_name::<SpriteGlyph>("TMPro", "TMP_SpriteGlyph").ok()?;
    sprite_glyph_ctor(glyph, None);
    let p = (glyph as *mut SpriteGlyph).cast::<u8>();
    set_field(p, 0x10, 0_u32);
    for (offset, value) in [(0x14, 56.0_f32), (0x18, 56.0), (0x1C, 0.0), (0x20, 56.0), (0x24, 56.0)] {
        set_field(p, offset, value);
    }
    for (offset, value) in [(0x28, 0_i32), (0x2C, 0), (0x30, 56), (0x34, 56)] {
        set_field(p, offset, value);
    }
    set_field(p, 0x38, 1.0_f32);
    set_field(p, 0x3C, 0_i32);
    set_field(p, 0x40, sprite);

    let character = il2cpp::instantiate_class_by_name::<SpriteCharacter>("TMPro", "TMP_SpriteCharacter").ok()?;
    sprite_character_ctor(character, 0xFFFE, asset, glyph, None);
    set_character_name(character, label, None);
    let glyph_list: *mut List<SpriteGlyph> = field(asset.cast(), 0xC0);
    let char_list: *mut List<SpriteCharacter> = field(asset.cast(), 0xB0);
    if glyph_list.is_null() || char_list.is_null() { return None; }
    (*glyph_list).add(glyph);
    (*char_list).add(character);
    update_lookup_tables(asset, None);

    // Enregistrement natif : la collection TMP conserve l'asset et son matériau en mémoire.
    let hash = 0x4000_0000_i32 + id as i32;
    register_sprite_asset(hash, asset, None);
    Some((asset, id))
}

#[skyline::hook(offset = 0x025DA3D0)]
fn create_skill_sprite_tag(skill: *const SkillData, method: OptionalMethod) -> *const Il2CppString {
    if skill.is_null() { return call_original!(skill, method); }
    let label = unsafe { get_icon_label(skill, None) };
    if label.is_null() { return call_original!(skill, method); }
    let label_text = unsafe { (*label).to_string() };
    if label_text.is_empty() { return call_original!(skill, method); }

    if let Some((name, _, _)) = assets().lock().unwrap().get(&label_text) {
        return Il2CppString::new_static(format!("<sprite=\"{name}\" name=\"{label_text}\">")) as *const Il2CppString;
    }

    let sprite = unsafe { get_skill_icon(label, None) };
    if sprite.is_null() { return call_original!(skill, method); }
    let texture = unsafe { sprite_texture(sprite, None) };
    // Les PNG Cobalt sont des textures autonomes de 56x56, contrairement à l'atlas vanilla.
    if texture.is_null()
        || unsafe { texture_width(texture, None) } != 56
        || unsafe { texture_height(texture, None) } != 56
    {
        return call_original!(skill, method);
    }

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let name = format!("{PREFIX}{id}");
    if let Some((asset, handle)) = unsafe { build_asset(&name, &*label, sprite, texture, id) } {
        assets().lock().unwrap().insert(label_text.clone(), (name.clone(), asset as usize, handle));
        println!("[class_skill_icon_sync] Icone TMP creee : {label_text}");
        return Il2CppString::new_static(format!("<sprite=\"{name}\" name=\"{label_text}\">")) as *const Il2CppString;
    }
    println!("[class_skill_icon_sync] Ressource TMP indisponible pour {label_text}");
    call_original!(skill, method)
}

#[skyline::hook(offset = 0x025D5020)]
fn sprite_asset_request(arg: i32, name: *const Il2CppString, method: OptionalMethod) -> *mut SpriteAsset {
    if !name.is_null() {
        let key = unsafe { (*name).to_string() };
        if key.starts_with(PREFIX) {
            let guard = assets().lock().unwrap();
            if let Some((_, address, _)) = guard.values().find(|(asset_name, _, _)| asset_name == &key) {
                return *address as *mut SpriteAsset;
            }
        }
    }
    call_original!(arg, name, method)
}

#[skyline::main(name = "class_skill_icon_sync")]
pub fn main() {
    skyline::install_hooks!(create_skill_sprite_tag, sprite_asset_request);
    println!("[class_skill_icon_sync] Plugin charge (Engage 2.0.0)");
}
