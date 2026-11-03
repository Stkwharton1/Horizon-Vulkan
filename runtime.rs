// VulkanMod, ported to Horizon by horizon-port from VulkanMod-0.6.8+26.3.jar.
// This file is both the bundle's Cargo library (its tests) and the module the bundle adds to
// the game (crate::game::port_vulkanmod). Put the mod's own logic here, in plain Rust
// that doesn't need the game, and call it from the helpers. No `//!` comments: module payloads
// can't hold inner attributes.

/// The mod's id (its settings folder), name, version and description.
pub const ID: &str = "vulkanmod";
pub const NAME: &str = "VulkanMod";
pub const VERSION: &str = "0.6.8+26.3";
pub const DESCRIPTION: &str = "Bring Vulkan to Minecraft!";

/// The mod's keys for Options > Mods: name, label, default key.
pub const KEYS: &[[&str; 3]] = &[];

/// The mod's own Java that doesn't need the game, translated to Rust by horizon-port.
/// Each Java method is here whole or not at all; the ones left out say why.
#[allow(dead_code, unused_variables, unused_mut, unused_parens, unused_imports, unreachable_code,
    non_snake_case, non_upper_case_globals, non_camel_case_types, clippy::all)]
pub mod java {
    use std::sync::{LazyLock, Mutex};

    /// A Java static that changes: one value behind a lock.
    pub struct JCell<T>(Mutex<T>);
    impl<T: Clone> JCell<T> {
        pub const fn new(v: T) -> Self {
            Self(Mutex::new(v))
        }
        pub fn get(&self) -> T {
            self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
        }
        pub fn set(&self, v: T) {
            *self.0.lock().unwrap_or_else(|e| e.into_inner()) = v;
        }
        pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
            f(&mut self.0.lock().unwrap_or_else(|e| e.into_inner()))
        }
    }

    /// Java's HashMap, LinkedHashMap and TreeMap. Keeps insertion order (LinkedHashMap's; HashMap's
    /// order is unspecified), or key order when made with `new_sorted` (TreeMap's).
    #[derive(Clone, Debug)]
    pub struct JMap<K, V> {
        index: std::collections::HashMap<K, usize>,
        entries: Vec<(K, V)>,
        sorted: bool,
    }
    /// Java's Map.equals: the same keys with equal values, in any order.
    impl<K: std::hash::Hash + Eq, V: PartialEq> PartialEq for JMap<K, V> {
        fn eq(&self, o: &Self) -> bool {
            self.entries.len() == o.entries.len()
                && self.entries.iter().all(|(k, v)| o.index.get(k).is_some_and(|&i| o.entries[i].1 == *v))
        }
    }
    impl<K, V> Default for JMap<K, V> {
        fn default() -> Self {
            Self { index: std::collections::HashMap::new(), entries: Vec::new(), sorted: false }
        }
    }
    impl<K: std::hash::Hash + Ord + Clone, V> JMap<K, V> {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn new_sorted() -> Self {
            Self { sorted: true, ..Self::default() }
        }
        pub fn from_iter(pairs: impl IntoIterator<Item = (K, V)>) -> Self {
            let mut m = Self::new();
            for (k, v) in pairs {
                m.insert(k, v);
            }
            m
        }
        pub fn from_iter_sorted(pairs: impl IntoIterator<Item = (K, V)>) -> Self {
            let mut m = Self::new_sorted();
            for (k, v) in pairs {
                m.insert(k, v);
            }
            m
        }
        pub fn len(&self) -> usize {
            self.entries.len()
        }
        pub fn is_empty(&self) -> bool {
            self.entries.is_empty()
        }
        pub fn clear(&mut self) {
            self.index.clear();
            self.entries.clear();
        }
        pub fn contains_key<Q: std::hash::Hash + Eq + ?Sized>(&self, k: &Q) -> bool
        where
            K: std::borrow::Borrow<Q>,
        {
            self.index.contains_key(k)
        }
        pub fn get<Q: std::hash::Hash + Eq + ?Sized>(&self, k: &Q) -> Option<&V>
        where
            K: std::borrow::Borrow<Q>,
        {
            self.index.get(k).map(|&i| &self.entries[i].1)
        }
        fn reindex(&mut self) {
            self.index.clear();
            for (i, (k, _)) in self.entries.iter().enumerate() {
                self.index.insert(k.clone(), i);
            }
        }
        fn slot(&mut self, k: K, v: V) -> usize {
            if self.sorted {
                let at = self.entries.partition_point(|e| e.0 < k);
                self.entries.insert(at, (k, v));
                self.reindex();
                at
            } else {
                self.index.insert(k.clone(), self.entries.len());
                self.entries.push((k, v));
                self.entries.len() - 1
            }
        }
        pub fn insert(&mut self, k: K, v: V) -> Option<V> {
            if let Some(&i) = self.index.get(&k) {
                return Some(std::mem::replace(&mut self.entries[i].1, v));
            }
            self.slot(k, v);
            None
        }
        pub fn remove<Q: std::hash::Hash + Eq + ?Sized>(&mut self, k: &Q) -> Option<V>
        where
            K: std::borrow::Borrow<Q>,
        {
            let i = *self.index.get(k)?;
            let (_, v) = self.entries.remove(i);
            self.reindex();
            Some(v)
        }
        /// computeIfAbsent and putIfAbsent: the value, made first when the key is missing.
        pub fn or_insert_with(&mut self, k: K, f: impl FnOnce() -> V) -> &mut V {
            let i = match self.index.get(&k) {
                Some(&i) => i,
                None => self.slot(k, f()),
            };
            &mut self.entries[i].1
        }
        pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
            self.entries.iter().map(|(k, v)| (k, v))
        }
        pub fn keys(&self) -> impl Iterator<Item = &K> {
            self.entries.iter().map(|e| &e.0)
        }
        pub fn values(&self) -> impl Iterator<Item = &V> {
            self.entries.iter().map(|e| &e.1)
        }
        pub fn clone_pairs(&self) -> Vec<(K, V)>
        where
            V: Clone,
        {
            self.entries.clone()
        }
    }

    /// Java's HashSet, LinkedHashSet and TreeSet, kept like JMap's keys.
    #[derive(Clone, Debug)]
    pub struct JSet<T>(JMap<T, ()>);
    impl<T: std::hash::Hash + Eq> PartialEq for JSet<T> {
        fn eq(&self, o: &Self) -> bool {
            self.0 == o.0
        }
    }
    impl<T> Default for JSet<T> {
        fn default() -> Self {
            Self(JMap::default())
        }
    }
    impl<T: std::hash::Hash + Ord + Clone> JSet<T> {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn new_sorted() -> Self {
            Self(JMap::new_sorted())
        }
        pub fn from_iter(items: impl IntoIterator<Item = T>) -> Self {
            let mut s = Self::new();
            for v in items {
                s.insert(v);
            }
            s
        }
        pub fn from_iter_sorted(items: impl IntoIterator<Item = T>) -> Self {
            let mut s = Self::new_sorted();
            for v in items {
                s.insert(v);
            }
            s
        }
        pub fn len(&self) -> usize {
            self.0.len()
        }
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
        pub fn clear(&mut self) {
            self.0.clear();
        }
        /// True when it wasn't there yet, as Java's add.
        pub fn insert(&mut self, v: T) -> bool {
            self.0.insert(v, ()).is_none()
        }
        pub fn contains<Q: std::hash::Hash + Eq + ?Sized>(&self, v: &Q) -> bool
        where
            T: std::borrow::Borrow<Q>,
        {
            self.0.contains_key(v)
        }
        pub fn remove<Q: std::hash::Hash + Eq + ?Sized>(&mut self, v: &Q) -> bool
        where
            T: std::borrow::Borrow<Q>,
        {
            self.0.remove(v).is_some()
        }
        pub fn iter(&self) -> impl Iterator<Item = &T> {
            self.0.keys()
        }
        pub fn clone_pairs(&self) -> Vec<T> {
            self.0.keys().cloned().collect()
        }
    }

    /// The mod's data folder in Horizon, set by the hooks (game_side.rs `ensure_init`) before the mod's code runs.
    pub static J_DATA_DIR: JCell<Option<std::path::PathBuf>> = JCell::new(None);
    /// FabricLoader's config folder: the mod's data folder in Horizon (./config until the game sets it).
    pub fn j_config_dir() -> std::path::PathBuf {
        J_DATA_DIR.get().unwrap_or_else(|| std::path::PathBuf::from("config"))
    }

    /// Minecraft's RandomSource / java.util.Random: other numbers than Java's, the same ranges.
    #[derive(Clone, Debug)]
    pub struct JRandom(u64);
    impl JRandom {
        pub fn new() -> Self {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9E37_79B9);
            Self(t | 1)
        }
        fn next(&mut self) -> u64 {
            let mut v = self.0;
            v ^= v << 13;
            v ^= v >> 7;
            v ^= v << 17;
            self.0 = v;
            v
        }
        /// 0 inclusive to n exclusive; Java throws on n <= 0.
        pub fn next_int(&mut self, n: i32) -> i32 {
            assert!(n > 0, "bound must be positive");
            (self.next() % n as u64) as i32
        }
        pub fn next_int_between(&mut self, lo: i32, hi: i32) -> i32 {
            lo + self.next_int(hi - lo)
        }
        pub fn next_float(&mut self) -> f32 {
            (self.next() >> 40) as f32 / (1u64 << 24) as f32
        }
        pub fn next_double(&mut self) -> f64 {
            (self.next() >> 11) as f64 / (1u64 << 53) as f64
        }
        pub fn next_boolean(&mut self) -> bool {
            self.next() & 1 == 1
        }
        pub fn next_gaussian(&mut self) -> f64 {
            let (u, v) = (self.next_double().max(1e-12), self.next_double());
            (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
        }
    }

    /// Minecraft's Identifier (`namespace:path`), which is only text.
    #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct JIdentifier {
        pub namespace: String,
        pub path: String,
    }
    impl JIdentifier {
        pub fn new(namespace: &str, path: &str) -> Self {
            Self { namespace: namespace.to_string(), path: path.to_string() }
        }
        /// `a:b`, or `minecraft:b` without a namespace.
        pub fn parse(s: &str) -> Self {
            match s.split_once(':') {
                Some((n, p)) => Self::new(n, p),
                None => Self::new("minecraft", s),
            }
        }
        /// Identifier.tryParse: None (null) when it has characters an identifier can't.
        pub fn try_parse(s: &str) -> Option<Self> {
            let id = Self::parse(s);
            Self::try_build(&id.namespace, &id.path)
        }
        pub fn try_build(namespace: &str, path: &str) -> Option<Self> {
            let ns_ok = namespace.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-'));
            let path_ok = path.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/'));
            if ns_ok && path_ok { Some(Self::new(namespace, path)) } else { None }
        }
    }
    impl std::fmt::Display for JIdentifier {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:{}", self.namespace, self.path)
        }
    }

    /// Java's String.equals.
    pub fn j_eq(a: &str, b: &str) -> bool {
        a == b
    }
    pub fn j_eq_ignore_case(a: &str, b: &str) -> bool {
        a.to_lowercase() == b.to_lowercase()
    }
    /// Java's String.length (in chars).
    pub fn j_len(s: &str) -> i32 {
        s.chars().count() as i32
    }
    pub fn j_char_at(s: &str, i: i32) -> char {
        s.chars().nth(i as usize).expect("charAt out of range")
    }
    /// Java's substring, by char.
    pub fn j_substring(s: &str, from: i32, to: Option<i32>) -> String {
        let n = s.chars().count() as i32;
        let to = to.unwrap_or(n);
        assert!(0 <= from && from <= to && to <= n, "substring out of range");
        s.chars().skip(from as usize).take((to - from) as usize).collect()
    }
    pub fn j_index_of(s: &str, sub: &str) -> i32 {
        s.find(sub).map_or(-1, |b| s[..b].chars().count() as i32)
    }
    pub fn j_index_of_char(s: &str, c: char) -> i32 {
        s.chars().position(|x| x == c).map_or(-1, |i| i as i32)
    }
    pub fn j_last_index_of(s: &str, sub: &str) -> i32 {
        s.rfind(sub).map_or(-1, |b| s[..b].chars().count() as i32)
    }
    /// Java's String.split on plain text: trailing empty parts dropped.
    pub fn j_split(s: &str, sep: &str) -> Vec<String> {
        let mut v: Vec<String> = s.split(sep).map(str::to_string).collect();
        while v.len() > 1 && v.last().is_some_and(|p| p.is_empty()) {
            v.pop();
        }
        v
    }
    /// Java's String.split(sep, limit) on plain text.
    pub fn j_split_limit(s: &str, sep: &str, limit: i32) -> Vec<String> {
        if limit == 0 {
            return j_split(s, sep);
        }
        if limit > 0 {
            return s.splitn(limit as usize, sep).map(str::to_string).collect();
        }
        s.split(sep).map(str::to_string).collect()
    }
    /// Java's String.compareTo: by UTF-16 unit, then by length.
    pub fn j_compare(a: &str, b: &str) -> i32 {
        let (x, y): (Vec<u16>, Vec<u16>) = (a.encode_utf16().collect(), b.encode_utf16().collect());
        for (p, q) in x.iter().zip(y.iter()) {
            if p != q {
                return *p as i32 - *q as i32;
            }
        }
        x.len() as i32 - y.len() as i32
    }
    /// Java's String.regionMatches, by char.
    pub fn j_region_matches(s: &str, ignore_case: bool, from: i32, other: &str, other_from: i32, len: i32) -> bool {
        let (a, b): (Vec<char>, Vec<char>) = (s.chars().collect(), other.chars().collect());
        if from < 0 || other_from < 0 || from as i64 + len as i64 > a.len() as i64 || other_from as i64 + len as i64 > b.len() as i64 {
            return false;
        }
        (0..len.max(0) as usize).all(|i| {
            let (p, q) = (a[from as usize + i], b[other_from as usize + i]);
            p == q || ignore_case && (j_upper_char(p) == j_upper_char(q) || j_lower_char(p) == j_lower_char(q))
        })
    }
    /// Java's Character.toLowerCase / toUpperCase (one char).
    pub fn j_lower_char(c: char) -> char {
        let mut l = c.to_lowercase();
        if l.len() == 1 { l.next().unwrap_or(c) } else { c }
    }
    pub fn j_upper_char(c: char) -> char {
        let mut u = c.to_uppercase();
        if u.len() == 1 { u.next().unwrap_or(c) } else { c }
    }
    /// Java's Arrays.copyOfRange: past the end is filled with defaults.
    pub fn j_copy_of_range<T: Clone + Default>(a: &[T], from: usize, to: usize) -> Vec<T> {
        assert!(from <= to && from <= a.len(), "copyOfRange out of range");
        let mut v: Vec<T> = a[from..to.min(a.len())].to_vec();
        v.resize(to - from, T::default());
        v
    }
    pub fn j_hash(s: &str) -> i32 {
        s.encode_utf16().fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(c as i32))
    }
    /// Java's Boolean.parseBoolean.
    pub fn j_parse_bool(s: &str) -> bool {
        s.eq_ignore_ascii_case("true")
    }
    /// A float as Java prints it in "..." + x (1.0, not 1).
    pub fn j_float(x: f64) -> String {
        if x.is_finite() && x == x.trunc() && x.abs() < 1e7 { format!("{x:.1}") } else { format!("{x}") }
    }
    /// Java's Math.round.
    pub fn j_round_f(x: f32) -> i32 {
        (x + 0.5).floor() as i32
    }
    pub fn j_round_d(x: f64) -> i64 {
        (x + 0.5).floor() as i64
    }
    /// Java's Math.clamp / Mth.clamp (no panic when min > max).
    pub fn j_clamp<T: PartialOrd>(v: T, min: T, max: T) -> T {
        if v < min { min } else if v > max { max } else { v }
    }
    pub fn j_lerp<T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<Output = T>>(d: T, a: T, b: T) -> T {
        a + d * (b - a)
    }
    pub fn j_signum_f(x: f32) -> f32 {
        if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { x }
    }
    pub fn j_signum_d(x: f64) -> f64 {
        if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { x }
    }
    pub fn j_wrap_degrees(x: f64) -> f64 {
        let mut v = x % 360.0;
        if v >= 180.0 {
            v -= 360.0;
        }
        if v < -180.0 {
            v += 360.0;
        }
        v
    }
    pub fn j_frac(x: f64) -> f64 {
        x - x.floor()
    }
    pub fn j_compare_f(a: f64, b: f64) -> i32 {
        a.partial_cmp(&b).map_or(0, |o| o as i32)
    }
    pub fn j_copy_of<T: Clone + Default>(a: &[T], n: usize) -> Vec<T> {
        let mut v: Vec<T> = a.iter().take(n).cloned().collect();
        v.resize(n, T::default());
        v
    }
    /// Java's System.currentTimeMillis.
    pub fn j_millis() -> i64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
    }
    /// Java's System.nanoTime: only differences mean anything.
    pub fn j_nanos() -> i64 {
        static START: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);
        START.elapsed().as_nanos() as i64
    }

    /// `Config` (net/vulkanmod/config/Config.java)
    pub mod config {
        use super::*;
        // CONFIG_PATH: not translated, a Path with no value (null in Java).
        // GSON: not translated, Gson.excludeFieldsWithModifiers.
        #[derive(Clone, Debug)]
        pub struct Config {
            pub video_mode: video_mode_set_video_mode::VideoMode,
            pub window_mode: i32,
            pub adv_culling: i32,
            pub indirect_draw: bool,
            pub unique_opaque_layer: bool,
            pub entity_culling: bool,
            pub device: i32,
            pub use_wayland: bool,
            pub ambient_occlusion: i32,
            pub frame_queue_size: i32,
            pub builder_threads: i32,
            pub back_face_culling: bool,
            pub texture_animations: bool,
        }
        impl Config {
            // Config.write(0): not translated, it uses Config.CONFIG_PATH, which isn't translated (a Path with no value (null in Java)).
        }
        // Config.load(1): not translated, it uses Config.CONFIG_PATH, which isn't translated (a Path with no value (null in Java)).
    }

    /// `GuiElement` (net/vulkanmod/config/gui/GuiElement.java)
    pub mod gui_element {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `GuiEventListener`. Its static parts are below.
        // GuiElement.setPosition(2): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.setPosition(4): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.resize(2): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.getX(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.getY(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.getWidth(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.getHeight(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.updateState(2): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.getHoverMultiplier(1): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.nextFocusPath(1): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.isMouseOver(2): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.getCurrentFocusPath(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.getRectangle(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.setFocused(1): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.isFocused(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.narrationPriority(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // GuiElement.updateNarration(1): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
    }

    /// `ModSettingsEntry` (net/vulkanmod/config/gui/ModSettingsEntry.java)
    pub mod mod_settings_entry {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `FormattedText`. Its static parts are below.
        // ModSettingsEntry.ModSettingsEntry(4): not translated, its class isn't translated (it uses Minecraft's `FormattedText`).
        // ModSettingsEntry.initPages(0): not translated, its class isn't translated (it uses Minecraft's `FormattedText`).
        // ModSettingsEntry.getPages(0): not translated, its class isn't translated (it uses Minecraft's `FormattedText`).
        // ModSettingsEntry.getIcon(0): not translated, its class isn't translated (it uses Minecraft's `FormattedText`).
        // ModSettingsEntry.runOnApply(0): not translated, its class isn't translated (it uses Minecraft's `FormattedText`).
    }

    /// `ModSettingsRegistry` (net/vulkanmod/config/gui/ModSettingsRegistry.java)
    pub mod mod_settings_registry {
        use super::*;
        // The class itself isn't translated: it uses ModSettingsEntry, which isn't translated (it uses Minecraft's `FormattedText`). Its static parts are below.
        // INSTANCE: not translated, it uses ModSettingsRegistry, which isn't translated (it uses ModSettingsEntry, which isn't translated (it uses Minecraft's `FormattedText`)).
        // ModSettingsRegistry.ModSettingsRegistry(0): not translated, its class isn't translated (it uses ModSettingsEntry, which isn't translated (it uses Minecraft's `FormattedText`)).
        // ModSettingsRegistry.addModEntry(1): not translated, its class isn't translated (it uses ModSettingsEntry, which isn't translated (it uses Minecraft's `FormattedText`)).
        // ModSettingsRegistry.getModEntries(0): not translated, its class isn't translated (it uses ModSettingsEntry, which isn't translated (it uses Minecraft's `FormattedText`)).
    }

    /// `OptionBlock` (net/vulkanmod/config/gui/OptionBlock.java)
    pub mod option_block {
        use super::*;
        // The class itself isn't translated: it uses Option, which isn't translated (it's generic). Its static parts are below.
        // OptionBlock.OptionBlock(2): not translated, its class isn't translated (it uses Option, which isn't translated (it's generic)).
        // OptionBlock.title(0): not translated, its class isn't translated (it uses Option, which isn't translated (it's generic)).
        // OptionBlock.options(0): not translated, its class isn't translated (it uses Option, which isn't translated (it's generic)).
    }

    /// `GuiRenderer` (net/vulkanmod/config/gui/render/GuiRenderer.java)
    pub mod gui_renderer {
        use super::*;
        // minecraft: not translated, it uses Minecraft's `Minecraft`.
        // guiGraphics: not translated, it uses Minecraft's `GuiGraphicsExtractor`.
        pub static POSE: LazyLock<JCell<JPoseStack>> = LazyLock::new(|| JCell::new(JPoseStack::new()));
        // bufferBuilder: not translated, it uses Minecraft's `BufferBuilder`.
        #[derive(Clone, Debug)]
        pub struct GuiRenderer {
        }
        impl GuiRenderer {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // GuiRenderer.enableScissor(4): not translated, it uses GuiRenderer.guiGraphics, which isn't translated (it uses Minecraft's `GuiGraphicsExtractor`).
        // GuiRenderer.disableScissor(0): not translated, it uses GuiRenderer.guiGraphics, which isn't translated (it uses Minecraft's `GuiGraphicsExtractor`).
        // GuiRenderer.fillBox(5): not translated, it calls GuiRenderer.fill, which isn't translated.
        // GuiRenderer.fill(5): not translated, it calls GuiRenderer.fill, which isn't translated.
        // GuiRenderer.fill(6): not translated, it uses GuiRenderer.guiGraphics, which isn't translated (it uses Minecraft's `GuiGraphicsExtractor`).
        // GuiRenderer.fillGradient(6): not translated, it calls GuiRenderer.fillGradient, which isn't translated.
        // GuiRenderer.fillGradient(7): not translated, it uses GuiRenderer.guiGraphics, which isn't translated (it uses Minecraft's `GuiGraphicsExtractor`).
        // GuiRenderer.renderBoxBorder(6): not translated, it calls GuiRenderer.renderBorder, which isn't translated.
        // GuiRenderer.renderBorder(6): not translated, it calls GuiRenderer.fill, which isn't translated.
        // GuiRenderer.drawString(5): not translated, it's overloaded with the same number of parameters.
        // GuiRenderer.drawString(5): not translated, it's overloaded with the same number of parameters.
        // GuiRenderer.drawString(6): not translated, it's overloaded with the same number of parameters.
        // GuiRenderer.drawString(6): not translated, it's overloaded with the same number of parameters.
        // GuiRenderer.drawCenteredString(5): not translated, it uses Minecraft's `Font`.
        // GuiRenderer.drawScrollingString(6): not translated, it uses Minecraft's `Font`.
        // GuiRenderer.getMaxTextWidth(2): not translated, it uses Minecraft's `Font`.
        // GuiRenderer.submitPolygon(4): not translated, it uses Minecraft's `RenderPipeline`.
    }

    /// `PolygonRenderState` (net/vulkanmod/config/gui/render/PolygonRenderState.java)
    pub mod polygon_render_state {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `GuiElementRenderState`. Its static parts are below.
        // PolygonRenderState.PolygonRenderState(7): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.PolygonRenderState(6): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.buildVertices(1): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.getBounds(3): not translated, it uses Minecraft's `ScreenRectangle`.
        // PolygonRenderState.pipeline(0): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.textureSetup(0): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.pose(0): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.vertices(0): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.col(0): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.scissorArea(0): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
        // PolygonRenderState.bounds(0): not translated, its class isn't translated (it implements Minecraft's `GuiElementRenderState`).
    }

    /// `SearchHelper` (net/vulkanmod/config/gui/util/SearchHelper.java)
    pub mod search_helper {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct SearchHelper {
        }
        impl SearchHelper {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // SearchHelper.matchesAnyValue(2): not translated, it's generic.
    }

    /// `VGuiConstants` (net/vulkanmod/config/gui/util/VGuiConstants.java)
    pub mod v_gui_constants {
        use super::*;
        pub static COLOR_WHITE: LazyLock<i32> = LazyLock::new(|| color_util_argb::pack(1.0f32, 1.0f32, 1.0f32, 1.0f32));
        pub static COLOR_BLACK: LazyLock<i32> = LazyLock::new(|| color_util_argb::pack(0.0f32, 0.0f32, 0.0f32, 1.0f32));
        pub static COLOR_GRAY: LazyLock<i32> = LazyLock::new(|| color_util_argb::pack(0.6f32, 0.6f32, 0.6f32, 1.0f32));
        pub static COLOR_RED: LazyLock<i32> = LazyLock::new(|| color_util_argb::pack(0.4f32, 0.05f32, 0.05f32, 0.8f32));
        pub const PAGE_BUTTON_WIDTH: i32 = 100;
        pub const WIDGET_HEIGHT: i32 = 20;
        pub const WIDGET_MARGIN: i32 = 5;
        #[derive(Clone, Debug)]
        pub struct VGuiConstants {
        }
        impl VGuiConstants {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
    }

    /// `VOptionList` (net/vulkanmod/config/gui/VOptionList.java)
    pub mod v_option_list {
        use super::*;
        // The class itself isn't translated: it extends `GuiElement`. Its static parts are below.
        // VOptionList.VOptionList(5): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.addButton(1): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.addAll(1): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.addEntry(1): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.clearEntries(0): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.updateScrollingState(2): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getScrollAmount(0): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.setScrollAmount(1): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getItemCount(0): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getFocused(0): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.setFocused(1): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.mouseClicked(2): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.mouseReleased(1): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.mouseDragged(3): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.mouseScrolled(4): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getMaxScroll(0): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getTotalLength(0): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getBottom(0): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getEntryAtPos(2): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.updateState(2): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.renderWidget(2): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getScrollbarPosition(0): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getHoveredWidget(2): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.renderList(2): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.getEntry(1): not translated, its class isn't translated (it extends `GuiElement`).
        // VOptionList.isValidClickButton(1): not translated, its class isn't translated (it extends `GuiElement`).
    }

    /// `VOptionList.Entry` (net/vulkanmod/config/gui/VOptionList.java)
    pub mod v_option_list_entry {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `GuiEventListener`. Its static parts are below.
        // VOptionList.Entry.Entry(3): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // VOptionList.Entry.render(5): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // VOptionList.Entry.getTotalHeight(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // VOptionList.Entry.mouseClicked(2): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // VOptionList.Entry.mouseReleased(1): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // VOptionList.Entry.mouseDragged(3): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // VOptionList.Entry.isFocused(0): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
        // VOptionList.Entry.setFocused(1): not translated, its class isn't translated (it implements Minecraft's `GuiEventListener`).
    }

    /// `VOptionScreen` (net/vulkanmod/config/gui/VOptionScreen.java)
    pub mod v_option_screen {
        use super::*;
        // The class itself isn't translated: it extends Minecraft's `Screen`. Its static parts are below.
        pub const MARGIN: i32 = 10;
        // VOptionScreen.VOptionScreen(2): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.init(0): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.initOptionsPages(0): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.createSearchField(0): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.buildLists(5): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.performSearch(1): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.buildPage(0): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.addButtonsWithSearchBar(0): not translated, its body couldn't be read: an assert.
        // VOptionScreen.mouseClicked(2): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.mouseReleased(1): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.onClose(0): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.extractRenderState(4): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.renderTooltip(3): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.getWidgetTooltip(1): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.updateState(0): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.setOptionList(1): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.undo(0): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.applyOptions(0): not translated, its class isn't translated (it extends Minecraft's `Screen`).
        // VOptionScreen.keyPressed(1): not translated, its class isn't translated (it extends Minecraft's `Screen`).
    }

    /// `CyclingOptionWidget` (net/vulkanmod/config/gui/widget/CyclingOptionWidget.java)
    pub mod cycling_option_widget {
        use super::*;
        // The class itself isn't translated: it extends `OptionWidget`. Its static parts are below.
        // CyclingOptionWidget.CyclingOptionWidget(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.setDimensions(4): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.renderControls(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.renderBars(0): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.setActive(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.onClick(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.onRelease(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.onDrag(4): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.setFocused(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // CyclingOptionWidget.isFocused(0): not translated, its class isn't translated (it extends `OptionWidget`).
    }

    /// `CyclingOptionWidget.Button` (net/vulkanmod/config/gui/widget/CyclingOptionWidget.java)
    pub mod cycling_option_widget_button {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Button {
            pub active_color: i32,
            pub hovered_color: i32,
            pub inactive_color: i32,
            pub x: i32,
            pub width: i32,
            pub active: bool,
            pub direction: cycling_option_widget_button_direction::Direction,
        }
        impl Button {
            pub fn new(direction: cycling_option_widget_button_direction::Direction) -> Self {
                let mut s = Self { active_color: color_util_argb::pack(1.0f32, 1.0f32, 1.0f32, 0.8f32), hovered_color: color_util_argb::pack(1.0f32, 1.0f32, 1.0f32, 1.0f32), inactive_color: color_util_argb::pack(0.3f32, 0.3f32, 0.3f32, 0.8f32), x: 0i32, width: 0i32, active: true, direction: direction };
                s.direction = direction;
                s
            }
            pub fn set_dimensions(&mut self, x: i32, width: i32) {
                self.x = x;
                self.width = width;
            }
            // CyclingOptionWidget.Button.isHovered(2): not translated, unknown static CyclingOptionWidget.this.
            pub fn set_status(&mut self, status: bool) {
                self.active = status;
            }
            // CyclingOptionWidget.Button.renderButton(2): not translated, it calls Button.isHovered, which isn't translated.
            // CyclingOptionWidget.Button.getVertices(1): not translated, unknown static CyclingOptionWidget.this.
        }
    }

    /// `CyclingOptionWidget.Button.Direction` (net/vulkanmod/config/gui/widget/CyclingOptionWidget.java)
    pub mod cycling_option_widget_button_direction {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum Direction {
            Left,
            Right,
        }
        impl Direction {
            pub const VALUES: [Direction; 2] = [Direction::Left, Direction::Right, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    Direction::Left => "LEFT",
                    Direction::Right => "RIGHT",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "LEFT" => Direction::Left,
                    "RIGHT" => Direction::Right,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `ModIconWidget` (net/vulkanmod/config/gui/widget/ModIconWidget.java)
    pub mod mod_icon_widget {
        use super::*;
        // The class itself isn't translated: it extends `VAbstractWidget`. Its static parts are below.
        // ModIconWidget.ModIconWidget(6): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // ModIconWidget.render(2): not translated, its class isn't translated (it extends `VAbstractWidget`).
    }

    /// `OptionWidget` (net/vulkanmod/config/gui/widget/OptionWidget.java)
    pub mod option_widget {
        use super::*;
        // The class itself isn't translated: it's generic. Its static parts are below.
        // OptionWidget.OptionWidget(2): not translated, its class isn't translated (it's generic).
        // OptionWidget.setDimensions(4): not translated, its class isn't translated (it's generic).
        // OptionWidget.render(2): not translated, its class isn't translated (it's generic).
        // OptionWidget.renderWidget(2): not translated, its class isn't translated (it's generic).
        // OptionWidget.renderControls(2): not translated, it has no body.
        // OptionWidget.onClick(2): not translated, it has no body.
        // OptionWidget.onRelease(2): not translated, it has no body.
        // OptionWidget.onDrag(4): not translated, it has no body.
        // OptionWidget.mouseDragged(3): not translated, its class isn't translated (it's generic).
        // OptionWidget.mouseClicked(2): not translated, its class isn't translated (it's generic).
        // OptionWidget.mouseReleased(1): not translated, its class isn't translated (it's generic).
        // OptionWidget.isMouseOver(2): not translated, its class isn't translated (it's generic).
        // OptionWidget.setFocused(1): not translated, its class isn't translated (it's generic).
        // OptionWidget.isFocused(0): not translated, its class isn't translated (it's generic).
        // OptionWidget.clicked(2): not translated, its class isn't translated (it's generic).
        // OptionWidget.getName(0): not translated, its class isn't translated (it's generic).
        // OptionWidget.getDisplayedValue(0): not translated, its class isn't translated (it's generic).
        // OptionWidget.updateDisplayedValue(0): not translated, its class isn't translated (it's generic).
        // OptionWidget.getTooltip(0): not translated, its class isn't translated (it's generic).
        // OptionWidget.getImpact(0): not translated, its class isn't translated (it's generic).
        // OptionWidget.narrationPriority(0): not translated, its class isn't translated (it's generic).
        // OptionWidget.updateNarration(1): not translated, its class isn't translated (it's generic).
    }

    /// `RangeOptionWidget` (net/vulkanmod/config/gui/widget/RangeOptionWidget.java)
    pub mod range_option_widget {
        use super::*;
        // The class itself isn't translated: it extends `OptionWidget`. Its static parts are below.
        // RangeOptionWidget.RangeOptionWidget(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.renderControls(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.onClick(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.keyPressed(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.setFocused(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.isFocused(0): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.setValueFromMouse(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.setValue(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.onDrag(4): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.applyNewValue(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.playDownSound(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // RangeOptionWidget.onRelease(2): not translated, its class isn't translated (it extends `OptionWidget`).
    }

    /// `SwitchOptionWidget` (net/vulkanmod/config/gui/widget/SwitchOptionWidget.java)
    pub mod switch_option_widget {
        use super::*;
        // The class itself isn't translated: it extends `OptionWidget`. Its static parts are below.
        // SwitchOptionWidget.SwitchOptionWidget(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // SwitchOptionWidget.renderControls(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // SwitchOptionWidget.onClick(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // SwitchOptionWidget.onRelease(2): not translated, its class isn't translated (it extends `OptionWidget`).
        // SwitchOptionWidget.onDrag(4): not translated, its class isn't translated (it extends `OptionWidget`).
        // SwitchOptionWidget.updateDisplayedValue(0): not translated, its class isn't translated (it extends `OptionWidget`).
        // SwitchOptionWidget.setFocused(1): not translated, its class isn't translated (it extends `OptionWidget`).
        // SwitchOptionWidget.isFocused(0): not translated, its class isn't translated (it extends `OptionWidget`).
    }

    /// `VAbstractWidget` (net/vulkanmod/config/gui/widget/VAbstractWidget.java)
    pub mod v_abstract_widget {
        use super::*;
        // The class itself isn't translated: it extends `GuiElement` (net.vulkanmod.config.gui). Its static parts are below.
        // VAbstractWidget.setDimensions(4): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.setTextLayout(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.render(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.renderWidget(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.onClick(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.onRelease(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.onDrag(4): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.setActive(1): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.renderHovering(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.mouseClicked(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.clicked(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.mouseReleased(1): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.isValidClickButton(1): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.mouseDragged(3): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.updateState(2): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.playDownSound(1): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.getTooltip(0): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
        // VAbstractWidget.getImpact(0): not translated, its class isn't translated (it extends `GuiElement` (net.vulkanmod.config.gui)).
    }

    /// `VButtonWidget` (net/vulkanmod/config/gui/widget/VButtonWidget.java)
    pub mod v_button_widget {
        use super::*;
        // The class itself isn't translated: it extends `VAbstractWidget`. Its static parts are below.
        // VButtonWidget.VButtonWidget(6): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VButtonWidget.renderWidget(2): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VButtonWidget.onClick(2): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VButtonWidget.setSelected(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VButtonWidget.isVisible(0): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VButtonWidget.isActive(0): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VButtonWidget.setActive(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VButtonWidget.nextFocusPath(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
    }

    /// `VTextInputWidget` (net/vulkanmod/config/gui/widget/VTextInputWidget.java)
    pub mod v_text_input_widget {
        use super::*;
        // The class itself isn't translated: it extends `VAbstractWidget`. Its static parts are below.
        pub const CURSOR_BLINK_INTERVAL: i32 = 500;
        // VTextInputWidget.VTextInputWidget(6): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.renderWidget(2): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.keyPressed(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.getInput(0): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.setInput(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.setSelected(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.isVisible(0): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.isActive(0): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.setActive(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.nextFocusPath(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.mouseClicked(2): not translated, its class isn't translated (it extends `VAbstractWidget`).
        // VTextInputWidget.setFocused(1): not translated, its class isn't translated (it extends `VAbstractWidget`).
    }

    /// `CyclingOption` (net/vulkanmod/config/option/CyclingOption.java)
    pub mod cycling_option {
        use super::*;
        // The class itself isn't translated: it's generic. Its static parts are below.
        // CyclingOption.CyclingOption(4): not translated, its class isn't translated (it's generic).
        // CyclingOption.createWidget(0): not translated, its class isn't translated (it's generic).
        // CyclingOption.updateOption(3): not translated, its class isn't translated (it's generic).
        // CyclingOption.index(0): not translated, its class isn't translated (it's generic).
        // CyclingOption.setValues(1): not translated, its class isn't translated (it's generic).
        // CyclingOption.prevValue(0): not translated, its class isn't translated (it's generic).
        // CyclingOption.nextValue(0): not translated, its class isn't translated (it's generic).
        // CyclingOption.updateValue(0): not translated, its class isn't translated (it's generic).
        // CyclingOption.setNewValue(1): not translated, its class isn't translated (it's generic).
        // CyclingOption.findNewValueIndex(0): not translated, its class isn't translated (it's generic).
        // CyclingOption.getValues(0): not translated, its class isn't translated (it's generic).
    }

    /// `Option` (net/vulkanmod/config/option/Option.java)
    pub mod option {
        use super::*;
        // The class itself isn't translated: it's generic. Its static parts are below.
        // Option.Option(5): not translated, its class isn't translated (it's generic).
        // Option.Option(4): not translated, its class isn't translated (it's generic).
        // Option.Option(3): not translated, its class isn't translated (it's generic).
        // Option.setOnApply(1): not translated, its class isn't translated (it's generic).
        // Option.setValueSupplier(1): not translated, its class isn't translated (it's generic).
        // Option.setTranslator(1): not translated, its class isn't translated (it's generic).
        // Option.getTranslator(0): not translated, its class isn't translated (it's generic).
        // Option.setTooltip(1): not translated, its class isn't translated (it's generic).
        // Option.getImpact(0): not translated, its class isn't translated (it's generic).
        // Option.setImpact(1): not translated, its class isn't translated (it's generic).
        // Option.setActive(1): not translated, its class isn't translated (it's generic).
        // Option.createWidget(0): not translated, it has no body.
        // Option.getWidget(0): not translated, its class isn't translated (it's generic).
        // Option.setNewValue(1): not translated, its class isn't translated (it's generic).
        // Option.updateActiveState(0): not translated, its class isn't translated (it's generic).
        // Option.getName(0): not translated, its class isn't translated (it's generic).
        // Option.setOnChange(1): not translated, its class isn't translated (it's generic).
        // Option.setActivationFn(1): not translated, its class isn't translated (it's generic).
        // Option.isChanged(0): not translated, its class isn't translated (it's generic).
        // Option.apply(0): not translated, its class isn't translated (it's generic).
        // Option.resetValue(0): not translated, its class isn't translated (it's generic).
        // Option.getNewValue(0): not translated, its class isn't translated (it's generic).
        // Option.getDisplayedValue(0): not translated, its class isn't translated (it's generic).
        // Option.getTooltip(0): not translated, its class isn't translated (it's generic).
    }

    /// `OptionPage` (net/vulkanmod/config/option/OptionPage.java)
    pub mod option_page {
        use super::*;
        // The class itself isn't translated: it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic)). Its static parts are below.
        // OptionPage.OptionPage(2): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // OptionPage.createList(5): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // OptionPage.getOptionList(0): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // OptionPage.optionChanged(0): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // OptionPage.applyOptionChanges(0): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // OptionPage.updateOptionStates(0): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // OptionPage.resetToOriginalState(0): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // OptionPage.setOrder(1): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // OptionPage.getOrder(0): not translated, its class isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
    }

    /// `Options` (net/vulkanmod/config/option/Options.java)
    pub mod options {
        use super::*;
        pub static FULLSCREEN_DIRTY: JCell<bool> = JCell::new(false);
        // config: not translated, its value reads other changing statics.
        // minecraft: not translated, it uses Minecraft's `Minecraft`.
        // window: not translated, it uses Minecraft's `Window`.
        // mcOptions: not translated, it uses `Options`.
        #[derive(Clone, Debug)]
        pub struct Options {
        }
        impl Options {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // Options.getOptionPages(0): not translated, it uses OptionPage, which isn't translated (it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // Options.getVideoOpts(0): not translated, it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic)).
        // Options.getGraphicsOpts(0): not translated, it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic)).
        // Options.getOptimizationOpts(0): not translated, it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic)).
        // Options.getOtherOpts(0): not translated, it uses OptionBlock, which isn't translated (it uses Option, which isn't translated (it's generic)).
    }

    /// `Page` (net/vulkanmod/config/option/Page.java)
    pub mod page {
        use super::*;
        // The class itself isn't translated: it uses Block, which isn't translated (it uses Option, which isn't translated (it's generic)). Its static parts are below.
        // Page.Page(1): not translated, its class isn't translated (it uses Block, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // Page.of(1): not translated, it uses Page, which isn't translated (it uses Block, which isn't translated (it uses Option, which isn't translated (it's generic))).
        // Page.block(1): not translated, `this` used as a value.
    }

    /// `Page.Block` (net/vulkanmod/config/option/Page.java)
    pub mod page_block {
        use super::*;
        // The class itself isn't translated: it uses Option, which isn't translated (it's generic). Its static parts are below.
        // Page.Block.Block(2): not translated, its class isn't translated (it uses Option, which isn't translated (it's generic)).
        // Page.Block.add(1): not translated, its class isn't translated (it uses Option, which isn't translated (it's generic)).
        // Page.Block.done(0): not translated, its class isn't translated (it uses Option, which isn't translated (it's generic)).
        // Page.Block.build(0): not translated, its class isn't translated (it uses Option, which isn't translated (it's generic)).
    }

    /// `PerformanceImpact` (net/vulkanmod/config/option/PerformanceImpact.java)
    pub mod performance_impact {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // PerformanceImpact.PerformanceImpact(1): not translated, its class isn't translated (its constants carry values).
        // PerformanceImpact.component(0): not translated, its class isn't translated (its constants carry values).
    }

    /// `RangeOption` (net/vulkanmod/config/option/RangeOption.java)
    pub mod range_option {
        use super::*;
        // The class itself isn't translated: it extends `Option`. Its static parts are below.
        // RangeOption.RangeOption(7): not translated, its class isn't translated (it extends `Option`).
        // RangeOption.RangeOption(6): not translated, its class isn't translated (it extends `Option`).
        // RangeOption.createWidget(0): not translated, its class isn't translated (it extends `Option`).
        // RangeOption.getName(0): not translated, its class isn't translated (it extends `Option`).
        // RangeOption.getScaledValue(0): not translated, its class isn't translated (it extends `Option`).
        // RangeOption.setNewValueFromScaledFloat(1): not translated, its class isn't translated (it extends `Option`).
        // RangeOption.setNewValue(1): not translated, its class isn't translated (it extends `Option`).
        // RangeOption.getScaledNewValue(0): not translated, its class isn't translated (it extends `Option`).
        // RangeOption.computeScaledValue(1): not translated, its class isn't translated (it extends `Option`).
    }

    /// `SwitchOption` (net/vulkanmod/config/option/SwitchOption.java)
    pub mod switch_option {
        use super::*;
        // The class itself isn't translated: it extends `Option`. Its static parts are below.
        // SwitchOption.SwitchOption(3): not translated, its class isn't translated (it extends `Option`).
        // SwitchOption.createWidget(0): not translated, its class isn't translated (it extends `Option`).
    }

    /// `Platform` (net/vulkanmod/config/Platform.java)
    pub mod platform {
        use super::*;
        // OS: not translated, it uses Minecraft's `OS`.
        pub static ACTIVE_PLAT: JCell<i32> = JCell::new(0i32);
        pub static ACTIVE_DE: LazyLock<JCell<Option<String>>> = LazyLock::new(|| JCell::new(None));
        #[derive(Clone, Debug)]
        pub struct Platform {
        }
        impl Platform {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // Platform.init(0): not translated, it calls `GLFW` (org.lwjgl.glfw).glfwInitHint.
        // Platform.determineLinuxDisplayServer(0): not translated, it calls `System`.getenv.
        // Platform.getSupportedPlat(0): not translated, it uses `SystemUtils` (org.apache.commons.lang3).IS_OS_WINDOWS.
        pub fn get_string_from_plat() -> String {
            return match platform::ACTIVE_PLAT.get() {
                393216 => "ANDROID".to_string(),
                393217 => "WIN32".to_string(),
                393218 => "MACOS".to_string(),
                393219 => "WAYLAND".to_string(),
                393220 => "X11".to_string(),
                _ => {
                panic!("{}", format!("Unexpected value: {}", platform::ACTIVE_PLAT.get()));
                },
            };
        }
        // Platform.determineDE(0): not translated, it calls `System`.getenv.
        // Platform.isWindows(0): not translated, it uses Platform.OS, which isn't translated (it uses Minecraft's `OS`).
        // Platform.isLinux(0): not translated, it uses Platform.OS, which isn't translated (it uses Minecraft's `OS`).
        // Platform.isMacOS(0): not translated, it uses Platform.OS, which isn't translated (it uses Minecraft's `OS`).
        pub fn get_active_plat() -> i32 {
            return platform::ACTIVE_PLAT.get();
        }
        pub fn is_android() -> bool {
            return platform::ACTIVE_PLAT.get() == 393216;
        }
        pub fn is_way_land() -> bool {
            return platform::ACTIVE_PLAT.get() == 393219;
        }
        pub fn is_x11() -> bool {
            return platform::ACTIVE_PLAT.get() == 393220;
        }
        pub fn is_gnome() -> bool {
            return (platform::ACTIVE_DE.get().expect("null")).contains("gnome");
        }
        pub fn is_weston() -> bool {
            return (platform::ACTIVE_DE.get().expect("null")).contains("weston");
        }
        pub fn is_generic() -> bool {
            return (platform::ACTIVE_DE.get().expect("null")).contains("generic");
        }
    }

    /// `UpdateChecker` (net/vulkanmod/config/UpdateChecker.java)
    pub mod update_checker {
        use super::*;
        pub static UPDATE_AVAILABLE: JCell<bool> = JCell::new(false);
        #[derive(Clone, Debug)]
        pub struct UpdateChecker {
        }
        impl UpdateChecker {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // UpdateChecker.checkForUpdates(0): not translated, a lambda with a block body here.
        pub fn is_update_available() -> bool {
            return update_checker::UPDATE_AVAILABLE.get();
        }
    }

    /// `VideoMode` (net/vulkanmod/config/video/VideoMode.java)
    pub mod video_mode {
        use super::*;
        #[derive(Clone, Debug, PartialEq)]
        pub struct VideoMode {
            pub width: i32,
            pub height: i32,
            pub bit_depth: i32,
            pub refresh_rate: i32,
        }
        impl VideoMode {
            pub fn new(width: i32, height: i32, bit_depth: i32, refresh_rate: i32) -> Self {
                let mut s = Self { width: width, height: height, bit_depth: bit_depth, refresh_rate: refresh_rate };
                s.width = width;
                s.height = height;
                s.bit_depth = bit_depth;
                s.refresh_rate = refresh_rate;
                s
            }
            pub fn to_string(&self) -> String {
                return format!("{}\u{d7}{}{}", self.width, self.height, if self.refresh_rate > 0 { format!(" @ {}Hz", self.refresh_rate) } else { "".to_string() });
            }
            pub fn with_refresh_rate(&self, new_rate: i32) -> video_mode::VideoMode {
                return video_mode::VideoMode::new(self.width, self.height, self.bit_depth, new_rate);
            }
            pub fn width(&self) -> i32 {
                return self.width;
            }
            pub fn height(&self) -> i32 {
                return self.height;
            }
            pub fn bit_depth(&self) -> i32 {
                return self.bit_depth;
            }
            pub fn refresh_rate(&self) -> i32 {
                return self.refresh_rate;
            }
        }
    }

    /// `VideoModeManager` (net/vulkanmod/config/video/VideoModeManager.java)
    pub mod video_mode_manager {
        use super::*;
        // monitors: not translated, it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs).
        // monitorToVideoModeSets: not translated, it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs).
        // osVideoModes: not translated, it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs).
        pub static SELECTED_MONITOR: JCell<i64> = JCell::new(0i64);
        pub static SELECTED_VIDEO_MODE: LazyLock<JCell<Option<video_mode_set_video_mode::VideoMode>>> = LazyLock::new(|| JCell::new(None));
        #[derive(Clone, Debug)]
        pub struct VideoModeManager {
        }
        impl VideoModeManager {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // VideoModeManager.init(1): not translated, it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs).
        // VideoModeManager.addMonitorVideoModes(1): not translated, it uses VideoModeManager.monitorToVideoModeSets, which isn't translated (it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs)).
        // VideoModeManager.removeMonitor(1): not translated, it uses VideoModeManager.monitorToVideoModeSets, which isn't translated (it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs)).
        // VideoModeManager.applySelectedVideoMode(0): not translated, a nullable object kept in a changing static.
        // VideoModeManager.getVideoResolutions(0): not translated, it uses VideoModeManager.monitorToVideoModeSets, which isn't translated (it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs)).
        // VideoModeManager.getFirstAvailable(0): not translated, it uses VideoModeManager.monitorToVideoModeSets, which isn't translated (it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs)).
        // VideoModeManager.getOsVideoMode(0): not translated, it uses VideoModeManager.osVideoModes, which isn't translated (it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs)).
        // VideoModeManager.getCurrentVideoMode(1): not translated, it uses `GLFWVidMode` (org.lwjgl.glfw).
        // VideoModeManager.getVideoResolutions(1): not translated, it calls `GLFW` (org.lwjgl.glfw).glfwGetVideoModes.
        // VideoModeManager.getVideoModeSet(1): not translated, it uses VideoModeManager.monitorToVideoModeSets, which isn't translated (it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs)).
        // VideoModeManager.selectBestMonitor(1): not translated, it uses Minecraft's `Window`.
        // VideoModeManager.checkConfigVideoMode(1): not translated, it calls VideoModeManager.getFirstAvailable, which isn't translated.
        // VideoModeManager.findBestMonitor(1): not translated, it uses Minecraft's `Monitor`.
        // VideoModeManager.getMonitors(0): not translated, it uses `Long2ObjectMap` (it.unimi.dsi.fastutil.longs).
    }

    /// `VideoModeSet` (net/vulkanmod/config/video/VideoModeSet.java)
    pub mod video_mode_set {
        use super::*;
        pub static DUMMY: LazyLock<JCell<Option<video_mode_set::VideoModeSet>>> = LazyLock::new(|| JCell::new(None));
        #[derive(Clone, Debug)]
        pub struct VideoModeSet {
            pub width: i32,
            pub height: i32,
            pub bit_depth: i32,
            pub refresh_rates: Vec<i32>,
        }
        impl VideoModeSet {
            // VideoModeSet.VideoModeSet(3): not translated, it creates `ObjectArrayList` (it.unimi.dsi.fastutil.objects).
            pub fn get_refresh_rate(&self) -> i32 {
                return self.refresh_rates[0];
            }
            pub fn has_refresh_rate(&self, r: i32) -> bool {
                return self.refresh_rates.contains(&r);
            }
            pub fn get_refresh_rates(&self) -> Vec<i32> {
                return self.refresh_rates.to_vec();
            }
            pub fn add_refresh_rate(&mut self, rr: i32) {
                self.refresh_rates.push(rr);
            }
            pub fn to_string(&self) -> String {
                return format!("{} x {}", self.width, self.height);
            }
            // VideoModeSet.equals(1): not translated, it uses `Object`.
            pub fn get_video_mode_1(&self, refresh: i32) -> video_mode_set_video_mode::VideoMode {
                let mut idx: i32 = self.refresh_rates.iter().position(|e| *e == refresh).map_or(-1, |i| i as i32);
                if idx == (-1) {
                    idx = 0;
                }
                return video_mode_set_video_mode::VideoMode::new(self.width, self.height, self.bit_depth, self.refresh_rates[idx as usize]);
            }
            pub fn get_video_mode_0(&self) -> video_mode_set_video_mode::VideoMode {
                let refresh_rate: i32 = self.refresh_rates[((self.refresh_rates.len() as i32) - 1) as usize];
                return video_mode_set_video_mode::VideoMode::new(self.width, self.height, self.bit_depth, refresh_rate);
            }
        }
        // VideoModeSet.getDummy(0): not translated, it creates a VideoModeSet with a constructor that isn't translated.
    }

    /// `VideoModeSet.VideoMode` (net/vulkanmod/config/video/VideoModeSet.java)
    pub mod video_mode_set_video_mode {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct VideoMode {
            pub width: i32,
            pub height: i32,
            pub bit_depth: i32,
            pub refresh_rate: i32,
        }
        impl VideoMode {
            pub fn new(width: i32, height: i32, bit_depth: i32, refresh_rate: i32) -> Self {
                let mut s = Self { width: width, height: height, bit_depth: bit_depth, refresh_rate: refresh_rate };
                s.width = width;
                s.height = height;
                s.bit_depth = bit_depth;
                s.refresh_rate = refresh_rate;
                s
            }
            pub fn to_string(&self) -> String {
                return format!("VideoMode[width={}, height={}, bitDepth={}, refreshRate={}]", self.width, self.height, self.bit_depth, self.refresh_rate);
            }
        }
    }

    /// `WindowMode` (net/vulkanmod/config/video/WindowMode.java)
    pub mod window_mode {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // WindowMode.WindowMode(1): not translated, its class isn't translated (its constants carry values).
        // WindowMode.fromValue(1): not translated, it uses WindowMode, which isn't translated (its constants carry values).
        // WindowMode.getComponentName(1): not translated, it uses WindowMode, which isn't translated (its constants carry values).
    }

    /// `GlUtil` (net/vulkanmod/gl/GlUtil.java)
    pub mod gl_util {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct GlUtil {
        }
        impl GlUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // GlUtil.extToShaderKind(1): not translated, it uses ShaderKind, which isn't translated (its constants carry values).
        // GlUtil.RGBtoRGBA_buffer(1): not translated, it uses `ByteBuffer` (java.nio).
        // GlUtil.BGRAtoRGBA_buffer(1): not translated, it uses `ByteBuffer` (java.nio).
        // GlUtil.vulkanFormat(2): not translated, a switch case that falls through.
        pub fn vulkan_format_1(gl_internal_format: i32) -> i32 {
            return match gl_internal_format {
                6402 | 33190 | 36012 => vulkan::get_default_depth_format(),
                33639 => 41,
                _ => {
                panic!("{}", format!("Unexpected value: {}", gl_internal_format));
                },
            };
        }
        pub fn get_gl_format(v_format: i32) -> i32 {
            return match v_format {
                9 => 6403,
                16 => 33319,
                37 => 6408,
                44 => 32993,
                _ => {
                panic!("{}", format!("Unexpected value: {}", v_format));
                },
            };
        }
    }

    /// `VkGlBuffer` (net/vulkanmod/gl/VkGlBuffer.java)
    pub mod vk_gl_buffer {
        use super::*;
        // The class itself isn't translated: it uses `ByteBuffer` (java.nio). Its static parts are below.
        pub static ID_COUNTER: JCell<i32> = JCell::new(1);
        // map: not translated, it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints).
        pub static BOUND_ID: JCell<i32> = JCell::new(0);
        // boundBuffer: not translated, it uses VkGlBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // arrayBufferBound: not translated, it uses VkGlBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // pixelPackBufferBound: not translated, it uses VkGlBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // pixelUnpackBufferBound: not translated, it uses VkGlBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VkGlBuffer.glGenBuffers(0): not translated, it uses VkGlBuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlBuffer.glBindBuffer(2): not translated, it uses VkGlBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VkGlBuffer.glBufferData(3): not translated, it's overloaded with the same number of parameters.
        // VkGlBuffer.glBufferData(3): not translated, it's overloaded with the same number of parameters.
        // VkGlBuffer.glMapBuffer(2): not translated, it uses `ByteBuffer` (java.nio).
        pub fn gl_unmap_buffer(i: i32) -> bool {
            return true;
        }
        // VkGlBuffer.glDeleteBuffers(1): not translated, it's overloaded with the same number of parameters.
        // VkGlBuffer.glDeleteBuffers(1): not translated, it's overloaded with the same number of parameters.
        // VkGlBuffer.getPixelUnpackBufferBound(0): not translated, it uses VkGlBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VkGlBuffer.getPixelPackBufferBound(0): not translated, it uses VkGlBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VkGlBuffer.checkTarget(1): not translated, String.formatted with 1 values.
        // VkGlBuffer.VkGlBuffer(1): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // VkGlBuffer.allocate(1): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // VkGlBuffer.getData(0): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // VkGlBuffer.freeData(0): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
    }

    /// `VkGlFramebuffer` (net/vulkanmod/gl/VkGlFramebuffer.java)
    pub mod vk_gl_framebuffer {
        use super::*;
        // The class itself isn't translated: it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)). Its static parts are below.
        pub static ID_COUNTER: JCell<i32> = JCell::new(1);
        // map: not translated, it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints).
        // boundFramebuffer: not translated, it uses VkGlFramebuffer, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // readFramebuffer: not translated, it uses VkGlFramebuffer, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.resetBoundFramebuffer(0): not translated, it uses VkGlFramebuffer, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.beginRendering(1): not translated, it uses VkGlFramebuffer, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.genFramebufferId(0): not translated, it uses VkGlFramebuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlFramebuffer.bindFramebuffer(2): not translated, it uses MainPass, which isn't translated (it's an interface).
        // VkGlFramebuffer.deleteFramebuffer(1): not translated, it uses VkGlFramebuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlFramebuffer.framebufferTexture2D(5): not translated, a nullable object kept in a changing static.
        // VkGlFramebuffer.framebufferRenderbuffer(4): not translated, a nullable object kept in a changing static.
        // VkGlFramebuffer.glBlitFramebuffer(10): not translated, a nullable object kept in a changing static.
        pub fn gl_check_framebuffer_status(target: i32) -> i32 {
            return 36053;
        }
        // VkGlFramebuffer.getBoundFramebuffer(0): not translated, it uses VkGlFramebuffer, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.getFramebuffer(1): not translated, it uses VkGlFramebuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlFramebuffer.VkGlFramebuffer(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.beginRendering(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.setAttachmentTexture(2): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.setAttachmentRenderbuffer(2): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.setAttachmentImage(2): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.setLevel(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.setColorAttachment(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.setDepthAttachment(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.create(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.getFramebuffer(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.getRenderPass(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // VkGlFramebuffer.cleanUp(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
    }

    /// `VkGlProgram` (net/vulkanmod/gl/VkGlProgram.java)
    pub mod vk_gl_program {
        use super::*;
        // The class itself isn't translated: it uses Pipeline, which isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))). Its static parts are below.
        pub static ID_COUNTER: JCell<i32> = JCell::new(1);
        // map: not translated, it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints).
        pub static BOUND_PROGRAM_ID: JCell<i32> = JCell::new(0);
        // boundProgram: not translated, it uses VkGlProgram, which isn't translated (it uses Pipeline, which isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio)))).
        // VkGlProgram.getBoundProgram(0): not translated, it uses VkGlProgram, which isn't translated (it uses Pipeline, which isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio)))).
        // VkGlProgram.getProgram(1): not translated, it uses VkGlProgram.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlProgram.genProgramId(0): not translated, it uses VkGlProgram.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlProgram.glUseProgram(1): not translated, it uses VkGlProgram.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlProgram.VkGlProgram(1): not translated, its class isn't translated (it uses Pipeline, which isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio)))).
        // VkGlProgram.bindPipeline(1): not translated, it stores a second reference to an object (Rust would need it copied or shared).
        // VkGlProgram.getPipeline(0): not translated, its class isn't translated (it uses Pipeline, which isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio)))).
    }

    /// `VkGlRenderbuffer` (net/vulkanmod/gl/VkGlRenderbuffer.java)
    pub mod vk_gl_renderbuffer {
        use super::*;
        // The class itself isn't translated: it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)). Its static parts are below.
        pub static ID_COUNTER: JCell<i32> = JCell::new(1);
        // map: not translated, it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints).
        pub static BOUND_ID: JCell<i32> = JCell::new(0);
        // bound: not translated, it uses VkGlRenderbuffer, which isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlRenderbuffer.genId(0): not translated, it uses VkGlRenderbuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlRenderbuffer.bindRenderbuffer(2): not translated, it uses VkGlRenderbuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlRenderbuffer.deleteRenderbuffer(1): not translated, it uses VkGlRenderbuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlRenderbuffer.getRenderbuffer(1): not translated, it uses VkGlRenderbuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlRenderbuffer.renderbufferStorage(4): not translated, a nullable object kept in a changing static.
        // VkGlRenderbuffer.texParameteri(3): not translated, a nullable object kept in a changing static.
        // VkGlRenderbuffer.getTexLevelParameter(3): not translated, a nullable object kept in a changing static.
        // VkGlRenderbuffer.generateMipmap(1): not translated, a nullable object kept in a changing static.
        // VkGlRenderbuffer.setVulkanImage(2): not translated, it uses VkGlRenderbuffer.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlRenderbuffer.getBound(0): not translated, it uses VkGlRenderbuffer, which isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlRenderbuffer.VkGlRenderbuffer(1): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlRenderbuffer.allocateIfNeeded(3): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlRenderbuffer.allocateImage(3): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlRenderbuffer.updateSampler(0): not translated, String.formatted with 1 values.
        // VkGlRenderbuffer.uploadImage(1): not translated, it uses `ByteBuffer` (java.nio).
        // VkGlRenderbuffer.generateMipmaps(0): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlRenderbuffer.setMaxLevel(1): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlRenderbuffer.setMaxLod(1): not translated, it calls VkGlRenderbuffer.updateSampler, which isn't translated.
        // VkGlRenderbuffer.setMagFilter(1): not translated, it calls VkGlRenderbuffer.updateSampler, which isn't translated.
        // VkGlRenderbuffer.setMinFilter(1): not translated, it calls VkGlRenderbuffer.updateSampler, which isn't translated.
        // VkGlRenderbuffer.getVulkanImage(0): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlRenderbuffer.setVulkanImage(1): not translated, it stores a second reference to an object (Rust would need it copied or shared).
    }

    /// `VkGlShader` (net/vulkanmod/gl/VkGlShader.java)
    pub mod vk_gl_shader {
        use super::*;
        pub static ID_COUNTER: JCell<i32> = JCell::new(1);
        // map: not translated, it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints).
        pub static BOUND_TEXTURE_ID: JCell<i32> = JCell::new(0);
        #[derive(Clone, Debug)]
        pub struct VkGlShader {
            pub id: i32,
            pub r#type: i32,
            pub source: Option<String>,
        }
        impl VkGlShader {
            pub fn new(id: i32, r#type: i32) -> Self {
                let mut s = Self { id: id, r#type: r#type, source: None };
                s.id = id;
                s.r#type = r#type;
                s
            }
        }
        // VkGlShader.glCreateShader(1): not translated, `++` inside an expression.
        // VkGlShader.glDeleteShader(1): not translated, it uses VkGlShader.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlShader.glShaderSource(2): not translated, it uses VkGlShader.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        pub fn gl_compile_shader(i: i32) {
        }
        pub fn gl_get_shaderi(i: i32, j: i32) -> i32 {
            return 0;
        }
    }

    /// `VkGlTexture` (net/vulkanmod/gl/VkGlTexture.java)
    pub mod vk_gl_texture {
        use super::*;
        // The class itself isn't translated: it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)). Its static parts are below.
        pub static ID_COUNTER: JCell<i32> = JCell::new(1);
        // map: not translated, it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints).
        pub static BOUND_TEXTURE_ID: JCell<i32> = JCell::new(0);
        // boundTexture: not translated, it uses VkGlTexture, which isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        pub static ACTIVE_TEXTURE: JCell<i32> = JCell::new(0);
        pub static UNPACK_ROW_LENGTH: JCell<i32> = JCell::new(0i32);
        pub static UNPACK_SKIP_ROWS: JCell<i32> = JCell::new(0i32);
        pub static UNPACK_SKIP_PIXELS: JCell<i32> = JCell::new(0i32);
        // VkGlTexture.bindIdToImage(2): not translated, it uses VkGlTexture.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlTexture.genTextureId(0): not translated, it uses VkGlTexture.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlTexture.bindTexture(1): not translated, it uses VkGlTexture.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlTexture.glDeleteTextures(1): not translated, it's overloaded with the same number of parameters.
        // VkGlTexture.glDeleteTextures(1): not translated, it's overloaded with the same number of parameters.
        // VkGlTexture.getTexture(1): not translated, it uses VkGlTexture.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        pub fn active_texture(i: i32) {
            vk_gl_texture::ACTIVE_TEXTURE.set(i - 33984);
            v_texture_selector::set_active_texture(vk_gl_texture::ACTIVE_TEXTURE.get());
        }
        // VkGlTexture.texImage2D(9): not translated, it's overloaded with the same number of parameters.
        // VkGlTexture.texImage2D(9): not translated, it's overloaded with the same number of parameters.
        pub fn check_params(level: i32, width: i32, height: i32) -> bool {
            return (width == 0) || (height == 0);
        }
        // VkGlTexture.texSubImage2D(9): not translated, it's overloaded with the same number of parameters.
        // VkGlTexture.getByteBuffer(3): not translated, it uses `ByteBuffer` (java.nio).
        // VkGlTexture.texSubImage2D(9): not translated, it's overloaded with the same number of parameters.
        // VkGlTexture.texParameteri(3): not translated, a nullable object kept in a changing static.
        // VkGlTexture.getTexParameteri(2): not translated, a nullable object kept in a changing static.
        // VkGlTexture.getTexLevelParameter(3): not translated, a nullable object kept in a changing static.
        pub fn pixel_store_i(p_name: i32, value: i32) {
            match p_name {
                3314 => {
                    vk_gl_texture::UNPACK_ROW_LENGTH.set(value);
                }
                3315 => {
                    vk_gl_texture::UNPACK_SKIP_ROWS.set(value);
                }
                3316 => {
                    vk_gl_texture::UNPACK_SKIP_PIXELS.set(value);
                }
                _ => {}
            }
        }
        pub fn generate_mipmap(target: i32) {
            if target != 3553 {
                panic!("{}", "target != GL_TEXTURE_2D not supported");
            }
        }
        // VkGlTexture.getTexImage(5): not translated, a nullable object kept in a changing static.
        // VkGlTexture.setVulkanImage(2): not translated, it uses VkGlTexture.map, which isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // VkGlTexture.getBoundTexture(0): not translated, it uses VkGlTexture, which isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlTexture.VkGlTexture(1): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlTexture.updateParams(5): not translated, it calls GlUtil.vulkanFormat, which isn't translated.
        // VkGlTexture.allocateIfNeeded(0): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlTexture.allocateImage(3): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlTexture.updateSampler(0): not translated, String.formatted with 1 values.
        // VkGlTexture.uploadSubImage(7): not translated, it uses `ByteBuffer` (java.nio).
        // VkGlTexture.generateMipmaps(0): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlTexture.setMaxLevel(1): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlTexture.setMaxLod(1): not translated, it calls VkGlTexture.updateSampler, which isn't translated.
        // VkGlTexture.setMagFilter(1): not translated, it calls VkGlTexture.updateSampler, which isn't translated.
        // VkGlTexture.setMinFilter(1): not translated, it calls VkGlTexture.updateSampler, which isn't translated.
        // VkGlTexture.setClamp(1): not translated, it calls VkGlTexture.updateSampler, which isn't translated.
        // VkGlTexture.getVulkanImage(0): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // VkGlTexture.setVulkanImage(1): not translated, it stores a second reference to an object (Rust would need it copied or shared).
    }

    /// `Initializer` (net/vulkanmod/Initializer.java)
    pub mod initializer {
        use super::*;
        // The class itself isn't translated: it implements Fabric's `ClientModInitializer`. Its static parts are below.
        // LOGGER: the logger. Log calls become comments.
        pub static VERSION: LazyLock<JCell<Option<String>>> = LazyLock::new(|| JCell::new(None));
        pub static CONFIG: LazyLock<JCell<Option<config::Config>>> = LazyLock::new(|| JCell::new(None));
        // Initializer.onInitializeClient(0): not translated, its class isn't translated (it implements Fabric's `ClientModInitializer`).
        // Initializer.loadConfig(1): not translated, it calls Config.load, which isn't translated.
        pub fn get_version() -> Option<String> {
            return initializer::VERSION.get();
        }
    }

    /// `BiomeManagerExtended` (net/vulkanmod/interfaces/biome/BiomeManagerExtended.java)
    pub mod biome_manager_extended {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // BiomeManagerExtended.of(1): not translated, it uses BiomeManagerExtended, which isn't translated (it's an interface).
        // BiomeManagerExtended.getBiomeZoomSeed(0): not translated, it has no body.
    }

    /// `BlockColorsExtended` (net/vulkanmod/interfaces/color/BlockColorsExtended.java)
    pub mod block_colors_extended {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // BlockColorsExtended.from(1): not translated, it uses BlockColorsExtended, which isn't translated (it's an interface).
        // BlockColorsExtended.getColorResolverMap(0): not translated, it has no body.
    }

    /// `ExtendedRenderType` (net/vulkanmod/interfaces/ExtendedRenderType.java)
    pub mod extended_render_type {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // ExtendedRenderType.getTerrainRenderType(0): not translated, it has no body.
    }

    /// `ExtendedVertexBuilder` (net/vulkanmod/interfaces/ExtendedVertexBuilder.java)
    pub mod extended_vertex_builder {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // ExtendedVertexBuilder.of(1): not translated, it uses ExtendedVertexBuilder, which isn't translated (it's an interface).
        // ExtendedVertexBuilder.canUseFastVertex(0): not translated, its class isn't translated (it's an interface).
        // ExtendedVertexBuilder.vertex(9): not translated, it has no body.
        // ExtendedVertexBuilder.vertex(7): not translated, its class isn't translated (it's an interface).
    }

    /// `FrustumMixed` (net/vulkanmod/interfaces/FrustumMixed.java)
    pub mod frustum_mixed {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // FrustumMixed.customFrustum(0): not translated, it has no body.
    }

    /// `ModelPartCubeMixed` (net/vulkanmod/interfaces/ModelPartCubeMixed.java)
    pub mod model_part_cube_mixed {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // ModelPartCubeMixed.getCubeModel(0): not translated, it has no body.
    }

    /// `ExtendedRenderPipeline` (net/vulkanmod/interfaces/shader/ExtendedRenderPipeline.java)
    pub mod extended_render_pipeline {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // ExtendedRenderPipeline.of(1): not translated, it uses ExtendedRenderPipeline, which isn't translated (it's an interface).
        // ExtendedRenderPipeline.setPipeline(1): not translated, it has no body.
        // ExtendedRenderPipeline.setProgram(1): not translated, it has no body.
        // ExtendedRenderPipeline.getPipeline(0): not translated, it has no body.
        // ExtendedRenderPipeline.getProgram(0): not translated, it has no body.
    }

    /// `VertexFormatMixed` (net/vulkanmod/interfaces/VertexFormatMixed.java)
    pub mod vertex_format_mixed {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // VertexFormatMixed.getOffset(1): not translated, it has no body.
        // VertexFormatMixed.getFastList(0): not translated, it has no body.
    }

    /// `VisibilitySetExtended` (net/vulkanmod/interfaces/VisibilitySetExtended.java)
    pub mod visibility_set_extended {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // VisibilitySetExtended.getVisibility(0): not translated, it has no body.
    }

    /// `VoxelShapeExtended` (net/vulkanmod/interfaces/VoxelShapeExtended.java)
    pub mod voxel_shape_extended {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // VoxelShapeExtended.getCornerOcclusion(0): not translated, it has no body.
    }

    /// `MixinPlugin` (net/vulkanmod/mixin/MixinPlugin.java)
    pub mod mixin_plugin {
        use super::*;
        // The class itself isn't translated: it implements `IMixinConfigPlugin` (org.spongepowered.asm.mixin.extensibility). Its static parts are below.
        // MixinPlugin.onLoad(1): not translated, its class isn't translated (it implements `IMixinConfigPlugin` (org.spongepowered.asm.mixin.extensibility)).
        // MixinPlugin.getRefMapperConfig(0): not translated, its class isn't translated (it implements `IMixinConfigPlugin` (org.spongepowered.asm.mixin.extensibility)).
        // MixinPlugin.shouldApplyMixin(2): not translated, its class isn't translated (it implements `IMixinConfigPlugin` (org.spongepowered.asm.mixin.extensibility)).
        // MixinPlugin.acceptTargets(2): not translated, its class isn't translated (it implements `IMixinConfigPlugin` (org.spongepowered.asm.mixin.extensibility)).
        // MixinPlugin.getMixins(0): not translated, its class isn't translated (it implements `IMixinConfigPlugin` (org.spongepowered.asm.mixin.extensibility)).
        // MixinPlugin.preApply(4): not translated, its class isn't translated (it implements `IMixinConfigPlugin` (org.spongepowered.asm.mixin.extensibility)).
        // MixinPlugin.postApply(4): not translated, its class isn't translated (it implements `IMixinConfigPlugin` (org.spongepowered.asm.mixin.extensibility)).
    }

    /// `VertexMultiConsumersM` (net/vulkanmod/mixin/vertex/VertexMultiConsumersM.java)
    pub mod vertex_multi_consumers_m {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct VertexMultiConsumersM {
        }
        impl VertexMultiConsumersM {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
    }

    /// `AreaBuffer` (net/vulkanmod/render/chunk/buffer/AreaBuffer.java)
    pub mod area_buffer {
        use super::*;
        // The class itself isn't translated: it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints). Its static parts are below.
        pub const DEBUG: bool = false;
        // LOGGER: the logger. Log calls become comments.
        // MEMORY_TYPE: not translated, it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)).
        // AreaBuffer.AreaBuffer(3): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.allocateBuffer(0): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.allocateSegment(1): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.freeSegment(1): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.upload(3): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.upload(3): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.findSegment(1): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.reallocate(1): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.moveUsedSegments(1): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.setSegmentFree(1): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.mergeSegments(2): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.updateDrawParams(1): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.getId(0): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.freeBuffer(0): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.fragmentation(0): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.checkSegments(0): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.getSize(0): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
        // AreaBuffer.getUsed(0): not translated, its class isn't translated (it uses `Int2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.ints)).
    }

    /// `AreaBuffer.Segment` (net/vulkanmod/render/chunk/buffer/AreaBuffer.java)
    pub mod area_buffer_segment {
        use super::*;
        // The class itself isn't translated: it holds an object of its own class (a linked structure). Its static parts are below.
        // AreaBuffer.Segment.Segment(2): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // AreaBuffer.Segment.getOffset(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // AreaBuffer.Segment.getSize(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // AreaBuffer.Segment.isFree(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // AreaBuffer.Segment.setFree(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // AreaBuffer.Segment.bindNext(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
    }

    /// `AreaBuffer.Usage` (net/vulkanmod/render/chunk/buffer/AreaBuffer.java)
    pub mod area_buffer_usage {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // AreaBuffer.Usage.Usage(1): not translated, its class isn't translated (its constants carry values).
    }

    /// `DrawBuffers` (net/vulkanmod/render/chunk/buffer/DrawBuffers.java)
    pub mod draw_buffers {
        use super::*;
        // The class itself isn't translated: it uses `Vector3i` (org.joml). Its static parts are below.
        pub const INDEX_SIZE: i32 = 2;
        pub static UNDEFINED_FACING_IDX: LazyLock<i32> = LazyLock::new(|| (quad_facing::QuadFacing::Undefined as i32));
        // POS_OFFSET: not translated, its value reads other changing statics.
        pub const CMD_STRIDE: i32 = 32;
        // cmdBufferPtr: not translated, it uses QuadFacing.COUNT, which isn't translated (it uses QuadFacing.VALUES, which isn't translated (it calls QuadFacing.values, which isn't in the mod's source)).
        // DrawBuffers.DrawBuffers(3): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.upload(3): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.updateUniformData(1): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.updateFadeUniform(3): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.getAreaBufferOrAlloc(1): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.getAreaBuffer(1): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.hasRenderType(1): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.encodeSectionOffset(3): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.updateChunkAreaOrigin(6): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.buildDrawBatchesIndirect(4): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.buildDrawBatchesDirect(3): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.getMask(2): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.bindBuffers(9): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.releaseBuffers(0): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.free(0): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.isAllocated(0): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.getVertexBuffers(0): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.getIndexBuffer(0): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
        // DrawBuffers.getDrawParamsPtr(0): not translated, its class isn't translated (it uses `Vector3i` (org.joml)).
    }

    /// `DrawParametersBuffer` (net/vulkanmod/render/chunk/buffer/DrawParametersBuffer.java)
    pub mod draw_parameters_buffer {
        use super::*;
        pub const INDEX_COUNT_OFFSET: i64 = 0i64;
        pub const FIRST_INDEX_OFFSET: i64 = 4i64;
        pub const VERTEX_OFFSET_OFFSET: i64 = 8i64;
        pub const BASE_INSTANCE_OFFSET: i64 = 12i64;
        pub const STRIDE: i64 = 16i64;
        pub const SECTIONS: i32 = 512;
        pub const FACINGS: i32 = 7;
        #[derive(Clone, Debug)]
        pub struct DrawParametersBuffer {
        }
        impl DrawParametersBuffer {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // DrawParametersBuffer.allocateBuffer(0): not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // DrawParametersBuffer.freeBuffer(1): not translated, it calls `MemoryUtil` (org.lwjgl.system).nmemAlignedFree.
        pub fn get_params_ptr(base_ptr: i64, section: i32, render_type: i32, facing: i32) -> i64 {
            return base_ptr + ((((((render_type * 512) + section) * 7) + facing) as i64) * 16i64);
        }
        // DrawParametersBuffer.resetParameters(1): not translated, it calls DrawParametersBuffer.setIndexCount, which isn't translated.
        // DrawParametersBuffer.setIndexCount(2): not translated, it calls MemoryAccess.memPutInt, which isn't translated.
        // DrawParametersBuffer.setFirstIndex(2): not translated, it calls MemoryAccess.memPutInt, which isn't translated.
        // DrawParametersBuffer.setVertexOffset(2): not translated, it calls MemoryAccess.memPutInt, which isn't translated.
        // DrawParametersBuffer.setBaseInstance(2): not translated, it calls MemoryAccess.memPutInt, which isn't translated.
        // DrawParametersBuffer.getIndexCount(1): not translated, it calls MemoryAccess.memGetInt, which isn't translated.
        // DrawParametersBuffer.getFirstIndex(1): not translated, it calls MemoryAccess.memGetInt, which isn't translated.
        // DrawParametersBuffer.getVertexOffset(1): not translated, it calls MemoryAccess.memGetInt, which isn't translated.
        // DrawParametersBuffer.getBaseInstance(1): not translated, it calls MemoryAccess.memGetInt, which isn't translated.
    }

    /// `UploadManager` (net/vulkanmod/render/chunk/buffer/UploadManager.java)
    pub mod upload_manager {
        use super::*;
        // The class itself isn't translated: it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs). Its static parts are below.
        // INSTANCE: not translated, it uses UploadManager, which isn't translated (it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs)).
        // UploadManager.createInstance(0): not translated, it uses UploadManager, which isn't translated (it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs)).
        // UploadManager.submitUploads(0): not translated, its class isn't translated (it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs)).
        // UploadManager.recordUpload(4): not translated, its class isn't translated (it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs)).
        // UploadManager.copyBuffer(2): not translated, its class isn't translated (it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs)).
        // UploadManager.copyBuffer(5): not translated, its class isn't translated (it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs)).
        // UploadManager.syncUploads(0): not translated, its class isn't translated (it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs)).
        // UploadManager.beginCommands(0): not translated, its class isn't translated (it uses `LongOpenHashSet` (it.unimi.dsi.fastutil.longs)).
    }

    /// `BiomeData` (net/vulkanmod/render/chunk/build/biome/BiomeData.java)
    pub mod biome_data {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `Biome`. Its static parts are below.
        pub const ZOOM_LENGTH: i32 = 4;
        pub const BIOMES_PER_SECTION: i32 = 64;
        pub const SIZE: i32 = 1728;
        // BiomeData.BiomeData(4): not translated, its class isn't translated (it uses Minecraft's `Biome`).
        // BiomeData.getBiomeData(5): not translated, its class isn't translated (it uses Minecraft's `Biome`).
        // BiomeData.getBiome(3): not translated, its class isn't translated (it uses Minecraft's `Biome`).
        // BiomeData.getSectionIdx(3): not translated, its class isn't translated (it uses Minecraft's `Biome`).
        // BiomeData.getOffset(4): not translated, its class isn't translated (it uses Minecraft's `Biome`).
        // BiomeData.computeCellOffset(4): not translated, it calls Minecraft's `LinearCongruentialGenerator`.next.
        pub fn get_fiddle(l: i64) -> f32 {
            let d: f32 = (((l >> 24).rem_euclid(1024i64)) as f32) * (9.765625e-4f32);
            return (d - 0.5f32) * 0.9f32;
        }
        pub fn get_relative_section_idx(x: i32, y: i32, z: i32) -> i32 {
            return ((((x * 3) * 3) + (y * 3)) + z) * 64;
        }
        pub fn get_relative_idx(x: i32, y: i32, z: i32) -> i32 {
            return (((x * 4) * 4) + (y * 4)) + z;
        }
    }

    /// `BlockColorRegistry` (net/vulkanmod/render/chunk/build/color/BlockColorRegistry.java)
    pub mod block_color_registry {
        use super::*;
        // The class itself isn't translated: it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects). Its static parts are below.
        // BlockColorRegistry.register(2): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
        // BlockColorRegistry.getTintSources(1): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
    }

    /// `BoxBlur` (net/vulkanmod/render/chunk/build/color/BoxBlur.java)
    pub mod box_blur {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct BoxBlur {
        }
        impl BoxBlur {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn blur(buffer: &[i32], temp: Option<Vec<i32>>, width: i32, filter_radius: i32) {
            box_blur::horizontal_blur(Some(buffer.to_vec()), temp.clone(), 0, width, filter_radius);
            box_blur::horizontal_blur(temp.clone(), Some(buffer.to_vec()), filter_radius, width, filter_radius);
        }
        pub fn horizontal_blur(src: Option<Vec<i32>>, dst: Option<Vec<i32>>, y0: i32, width: i32, filter_radius: i32) {
            let div: i32 = (filter_radius * 2) + 1;
            let x0: i32 = filter_radius;
            let total_width: i32 = (filter_radius * 2) + width;
            for y in y0..total_width {
                let mut r: i32 = 0;
                let mut g: i32 = 0;
                let mut b: i32 = 0;
                for x in 0..((x0 + 1) + filter_radius) {
                    let mut color: i32 = (src.as_ref().expect("null"))[box_blur::get_idx(x, y, total_width) as usize];
                    r += box_blur::unpack_r(color);
                    g += box_blur::unpack_g(color);
                    b += box_blur::unpack_b(color);
                }
                (dst.as_mut().expect("null"))[box_blur::get_idx(y, x0, total_width) as usize] = box_blur::pack_color(r, g, b, div);
                for x in (x0 + 1)..(x0 + width) {
                    let mut color: i32 = (src.as_ref().expect("null"))[(box_blur::get_idx((x - filter_radius) - 1, y, total_width)) as usize];
                    let var16: i32 = r - box_blur::unpack_r(color);
                    let var17: i32 = g - box_blur::unpack_g(color);
                    let var18: i32 = b - box_blur::unpack_b(color);
                    color = (src.as_ref().expect("null"))[box_blur::get_idx(x + filter_radius, y, total_width) as usize];
                    r = var16 + box_blur::unpack_r(color);
                    g = var17 + box_blur::unpack_g(color);
                    b = var18 + box_blur::unpack_b(color);
                    (dst.as_mut().expect("null"))[box_blur::get_idx(y, x, total_width) as usize] = box_blur::pack_color(r, g, b, div);
                }
            }
        }
        pub fn get_idx(x: i32, z: i32, width: i32) -> i32 {
            return x + (z * width);
        }
        pub fn unpack_r(color: i32) -> i32 {
            return (color >> 16) & 0xFF;
        }
        pub fn unpack_g(color: i32) -> i32 {
            return (color >> 8) & 0xFF;
        }
        pub fn unpack_b(color: i32) -> i32 {
            return color & 0xFF;
        }
        pub fn pack_color(r: i32, g: i32, b: i32, div: i32) -> i32 {
            return (((0xFF000000u32 as i32) | (((r / div) & 0xFF) << 16)) | (((g / div) & 0xFF) << 8)) | ((b / div) & 0xFF);
        }
    }

    /// `TintCache` (net/vulkanmod/render/chunk/build/color/TintCache.java)
    pub mod tint_cache {
        use super::*;
        // The class itself isn't translated: it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects). Its static parts are below.
        pub const SECTION_WIDTH: i32 = 16;
        pub const BOUNDARY_WIDTH: i32 = 16;
        pub const LAYER_COUNT: i32 = 48;
        // TintCache.TintCache(0): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
        // TintCache.init(5): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
        // TintCache.getColor(2): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
        // TintCache.addResolver(1): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
        // TintCache.allocateLayers(0): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
        // TintCache.calculateLayer(3): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
        // TintCache.applyBlur(1): not translated, its class isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
    }

    /// `TintCache.Layer` (net/vulkanmod/render/chunk/build/color/TintCache.java)
    pub mod tint_cache_layer {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Layer {
            pub invalidated: bool,
            pub values: Option<Vec<i32>>,
        }
        impl Layer {
            pub fn new() -> Self {
                let mut s = Self { invalidated: true, values: None };
                s
            }
            pub fn allocate(&mut self, size: i32) {
                self.values = Some(vec![0i32; size as usize]);
                self.invalidate();
            }
            pub fn invalidate(&mut self) {
                self.invalidated = true;
            }
            pub fn get_values(&self) -> Option<Vec<i32>> {
                return self.values.clone();
            }
        }
    }

    /// `GeometryHelper` (net/vulkanmod/render/chunk/build/frapi/helper/GeometryHelper.java)
    pub mod geometry_helper {
        use super::*;
        pub const CUBIC_FLAG: i32 = 1;
        pub const AXIS_ALIGNED_FLAG: i32 = 2;
        pub const LIGHT_FACE_FLAG: i32 = 4;
        pub const FLAG_BIT_COUNT: i32 = 3;
        pub const EPS_MIN: f32 = 1.0e-4f32;
        pub const EPS_MAX: f32 = 0.9999f32;
        #[derive(Clone, Debug)]
        pub struct GeometryHelper {
        }
        impl GeometryHelper {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // GeometryHelper.computeShapeFlags(1): not translated, it uses Fabric's `QuadView`.
        // GeometryHelper.isQuadParallelToFace(2): not translated, it uses Fabric's `QuadView`.
        // GeometryHelper.isParallelQuadOnFace(2): not translated, it uses Fabric's `QuadView`.
        // GeometryHelper.isQuadCubic(2): not translated, it uses Fabric's `QuadView`.
        // GeometryHelper.confirmSquareCorners(3): not translated, it uses Fabric's `QuadView`.
        // GeometryHelper.lightFace(1): not translated, it uses Fabric's `QuadView`.
        pub fn min(a: f32, b: f32, c: f32, d: f32) -> f32 {
            let x: f32 = if a < b { a } else { b };
            let y: f32 = if c < d { c } else { d };
            return if x < y { x } else { y };
        }
        pub fn max(a: f32, b: f32, c: f32, d: f32) -> f32 {
            let x: f32 = if a > b { a } else { b };
            let y: f32 = if c > d { c } else { d };
            return if x > y { x } else { y };
        }
        // GeometryHelper.longestAxis(1): not translated, it uses Minecraft's `Axis`.
        // GeometryHelper.longestAxis(3): not translated, it uses Minecraft's `Axis`.
    }

    /// `NormalHelper` (net/vulkanmod/render/chunk/build/frapi/helper/NormalHelper.java)
    pub mod normal_helper {
        use super::*;
        pub const PACK: f32 = 127.0f32;
        pub const UNPACK: f32 = 0.007874016f32;
        #[derive(Clone, Debug)]
        pub struct NormalHelper {
        }
        impl NormalHelper {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn pack_normal_4(mut x: f32, mut y: f32, mut z: f32, mut w: f32) -> i32 {
            x = j_clamp(x, -1.0f32, 1.0f32);
            y = j_clamp(y, -1.0f32, 1.0f32);
            z = j_clamp(z, -1.0f32, 1.0f32);
            w = j_clamp(w, -1.0f32, 1.0f32);
            return (((((x * 127.0f32) as i32) & 0xFF) | ((((y * 127.0f32) as i32) & 0xFF) << 8)) | ((((z * 127.0f32) as i32) & 0xFF) << 16)) | ((((w * 127.0f32) as i32) & 0xFF) << 24);
        }
        pub fn pack_normal_2(normal: JVec3, w: f32) -> i32 {
            return normal_helper::pack_normal_4(normal.x, normal.y, normal.z, w);
        }
        pub fn pack_normal_3(mut x: f32, mut y: f32, mut z: f32) -> i32 {
            x = j_clamp(x, -1.0f32, 1.0f32);
            y = j_clamp(y, -1.0f32, 1.0f32);
            z = j_clamp(z, -1.0f32, 1.0f32);
            return ((((x * 127.0f32) as i32) & 0xFF) | ((((y * 127.0f32) as i32) & 0xFF) << 8)) | ((((z * 127.0f32) as i32) & 0xFF) << 16);
        }
        pub fn pack_normal_1(normal: JVec3) -> i32 {
            return normal_helper::pack_normal_3(normal.x, normal.y, normal.z);
        }
        pub fn unpack_normal_x(packed_normal: i32) -> f32 {
            return (((packed_normal & 0xFF) as i8) as f32) * 0.007874016f32;
        }
        pub fn unpack_normal_y(packed_normal: i32) -> f32 {
            return ((((((packed_normal as u32) >> 8) as i32) & 0xFF) as i8) as f32) * 0.007874016f32;
        }
        pub fn unpack_normal_z(packed_normal: i32) -> f32 {
            return ((((((packed_normal as u32) >> 16) as i32) & 0xFF) as i8) as f32) * 0.007874016f32;
        }
        pub fn unpack_normal_w(packed_normal: i32) -> f32 {
            return ((((((packed_normal as u32) >> 24) as i32) & 0xFF) as i8) as f32) * 0.007874016f32;
        }
        pub fn unpack_normal(packed_normal: i32, target: Option<JVec3>) {
            JVec3::new(normal_helper::unpack_normal_x(packed_normal), normal_helper::unpack_normal_y(packed_normal), normal_helper::unpack_normal_z(packed_normal));
        }
        // NormalHelper.computeFaceNormal(2): not translated, it uses Fabric's `QuadView`.
        // NormalHelper.computePackedNormal(1): not translated, it uses ModelQuadView, which isn't translated (it's an interface).
        // NormalHelper.packedNormalFromDirection(1): not translated, it uses Minecraft's `Vec3i`.
        pub fn unpack_normal_to(packed_normal: i32, mut normal: JVec3) {
            { let joml_0 = JVec3::new(i32_s_norm::unpack_x(packed_normal), i32_s_norm::unpack_y(packed_normal), i32_s_norm::unpack_z(packed_normal)); normal = joml_0; joml_0 };
        }
    }

    /// `EncodingFormat` (net/vulkanmod/render/chunk/build/frapi/mesh/EncodingFormat.java)
    pub mod encoding_format {
        use super::*;
        pub const HEADER_BITS: i32 = 0;
        pub const HEADER_FACE_NORMAL: i32 = 1;
        pub const HEADER_TINT_INDEX: i32 = 2;
        pub const HEADER_TAG: i32 = 3;
        pub const HEADER_STRIDE: i32 = 4;
        pub const VERTEX_X: i32 = 4;
        pub const VERTEX_Y: i32 = encoding_format::VERTEX_X + 1;
        pub const VERTEX_Z: i32 = encoding_format::VERTEX_Y + 1;
        pub const VERTEX_COLOR: i32 = encoding_format::VERTEX_Z + 1;
        pub const VERTEX_U: i32 = encoding_format::VERTEX_COLOR + 1;
        pub const VERTEX_V: i32 = encoding_format::VERTEX_U + 1;
        pub const VERTEX_LIGHTMAP: i32 = encoding_format::VERTEX_V + 1;
        pub const VERTEX_NORMAL: i32 = encoding_format::VERTEX_LIGHTMAP + 1;
        pub const VERTEX_POSITION_BYTES: i32 = 4;
        pub const VERTEX_COLOR_BYTES: i32 = 4;
        pub const VERTEX_UV_BYTES: i32 = 4;
        pub const VERTEX_LIGHTMAP_BYTES: i32 = 4;
        pub const VERTEX_NORMAL_BYTES: i32 = 4;
        pub const VERTEX_COLOR_SIZE: i32 = 1;
        pub const VERTEX_UV_SIZE: i32 = 1;
        pub const VERTEX_LIGHTMAP_SIZE: i32 = 1;
        pub const VERTEX_NORMAL_SIZE: i32 = 1;
        pub const VERTEX_POSITION_SIZE: i32 = 1;
        pub const VERTEX_STRIDE: i32 = 8;
        pub const QUAD_STRIDE: i32 = 32;
        pub const QUAD_STRIDE_BYTES: i32 = encoding_format::QUAD_STRIDE * 4;
        pub const TOTAL_STRIDE: i32 = 4 + encoding_format::QUAD_STRIDE;
        pub static DIRECTION_COUNT: LazyLock<i32> = LazyLock::new(|| (cycling_option_widget_button_direction::Direction::VALUES.to_vec().len() as i32));
        pub static NULLABLE_DIRECTION_COUNT: LazyLock<i32> = LazyLock::new(|| (*encoding_format::DIRECTION_COUNT) + 1);
        // CHUNK_SECTION_LAYERS: not translated, it uses Minecraft's `ChunkSectionLayer`.
        // CHUNK_SECTION_LAYER_COUNT: not translated, it uses EncodingFormat.CHUNK_SECTION_LAYERS, which isn't translated (it uses Minecraft's `ChunkSectionLayer`).
        // ITEM_RENDER_TYPES: not translated, it uses Minecraft's `RenderType`.
        // ITEM_RENDER_TYPE_COUNT: not translated, it uses EncodingFormat.ITEM_RENDER_TYPES, which isn't translated (it uses Minecraft's `RenderType`).
        // TRI_STATES: not translated, it uses Fabric's `TriState`.
        // TRI_STATE_COUNT: not translated, it uses EncodingFormat.TRI_STATES, which isn't translated (it uses Fabric's `TriState`).
        // NULLABLE_FOIL_TYPES: not translated, it uses Minecraft's `FoilType`.
        // NULLABLE_FOIL_TYPE_COUNT: not translated, it uses EncodingFormat.NULLABLE_FOIL_TYPES, which isn't translated (it uses Minecraft's `FoilType`).
        // SHADE_MODES: not translated, it uses Fabric's `ShadeMode`.
        // SHADE_MODE_COUNT: not translated, it uses EncodingFormat.SHADE_MODES, which isn't translated (it uses Fabric's `ShadeMode`).
        // QUAD_ATLASES: not translated, it uses Fabric's `QuadAtlas`.
        // QUAD_ATLAS_COUNT: not translated, it uses EncodingFormat.QUAD_ATLASES, which isn't translated (it uses Fabric's `QuadAtlas`).
        // NULL_FOIL_TYPE_INDEX: not translated, it uses EncodingFormat.NULLABLE_FOIL_TYPE_COUNT, which isn't translated (it uses EncodingFormat.NULLABLE_FOIL_TYPES, which isn't translated (it uses Minecraft's `FoilType`)).
        // CULL_BIT_LENGTH: not translated, it calls Minecraft's `Mth`.ceillog2.
        // LIGHT_BIT_LENGTH: not translated, it calls Minecraft's `Mth`.ceillog2.
        pub const NORMALS_BIT_LENGTH: i32 = 4;
        pub const GEOMETRY_BIT_LENGTH: i32 = 3;
        // QUAD_ATLAS_BIT_LENGTH: not translated, it uses EncodingFormat.QUAD_ATLAS_COUNT, which isn't translated (it uses EncodingFormat.QUAD_ATLASES, which isn't translated (it uses Fabric's `QuadAtlas`)).
        // CHUNK_LAYER_BIT_LENGTH: not translated, it uses EncodingFormat.CHUNK_SECTION_LAYER_COUNT, which isn't translated (it uses EncodingFormat.CHUNK_SECTION_LAYERS, which isn't translated (it uses Minecraft's `ChunkSectionLayer`)).
        // ITEM_RENDER_TYPE_BIT_LENGTH: not translated, it uses EncodingFormat.ITEM_RENDER_TYPE_COUNT, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPES, which isn't translated (it uses Minecraft's `RenderType`)).
        pub const EMISSIVE_BIT_LENGTH: i32 = 1;
        pub const DIFFUSE_BIT_LENGTH: i32 = 1;
        // AO_BIT_LENGTH: not translated, it uses EncodingFormat.TRI_STATE_COUNT, which isn't translated (it uses EncodingFormat.TRI_STATES, which isn't translated (it uses Fabric's `TriState`)).
        // FOIL_TYPE_BIT_LENGTH: not translated, it uses EncodingFormat.NULLABLE_FOIL_TYPE_COUNT, which isn't translated (it uses EncodingFormat.NULLABLE_FOIL_TYPES, which isn't translated (it uses Minecraft's `FoilType`)).
        // SHADE_MODE_BIT_LENGTH: not translated, it uses EncodingFormat.SHADE_MODE_COUNT, which isn't translated (it uses EncodingFormat.SHADE_MODES, which isn't translated (it uses Fabric's `ShadeMode`)).
        pub const ANIMATED_BIT_LENGTH: i32 = 1;
        pub const CULL_BIT_OFFSET: i32 = 0;
        // LIGHT_BIT_OFFSET: not translated, it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2).
        // NORMALS_BIT_OFFSET: not translated, it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)).
        // GEOMETRY_BIT_OFFSET: not translated, it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))).
        // QUAD_ATLAS_BIT_OFFSET: not translated, it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))).
        // CHUNK_LAYER_BIT_OFFSET: not translated, it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))).
        // ITEM_RENDER_TYPE_BIT_OFFSET: not translated, it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))).
        // EMISSIVE_BIT_OFFSET: not translated, it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))))).
        // DIFFUSE_BIT_OFFSET: not translated, it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))))).
        // AO_BIT_OFFSET: not translated, it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))))))).
        // FOIL_TYPE_BIT_OFFSET: not translated, it uses EncodingFormat.AO_BIT_OFFSET, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))))))).
        // SHADE_MODE_BIT_OFFSET: not translated, it uses EncodingFormat.FOIL_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.AO_BIT_OFFSET, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))))))))).
        // ANIMATED_BIT_OFFSET: not translated, it uses EncodingFormat.SHADE_MODE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.FOIL_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.AO_BIT_OFFSET, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))))))))).
        // TOTAL_BIT_LENGTH: not translated, it uses EncodingFormat.ANIMATED_BIT_OFFSET, which isn't translated (it uses EncodingFormat.SHADE_MODE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.FOIL_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.AO_BIT_OFFSET, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))))))))))).
        // CULL_MASK: not translated, it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2).
        // LIGHT_MASK: not translated, it uses EncodingFormat.LIGHT_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2).
        // NORMALS_MASK: not translated, it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))).
        // GEOMETRY_MASK: not translated, it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))).
        // QUAD_ATLAS_MASK: not translated, it uses EncodingFormat.QUAD_ATLAS_BIT_LENGTH, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_COUNT, which isn't translated (it uses EncodingFormat.QUAD_ATLASES, which isn't translated (it uses Fabric's `QuadAtlas`))).
        // CHUNK_LAYER_MASK: not translated, it uses EncodingFormat.CHUNK_LAYER_BIT_LENGTH, which isn't translated (it uses EncodingFormat.CHUNK_SECTION_LAYER_COUNT, which isn't translated (it uses EncodingFormat.CHUNK_SECTION_LAYERS, which isn't translated (it uses Minecraft's `ChunkSectionLayer`))).
        // ITEM_RENDER_TYPE_MASK: not translated, it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_LENGTH, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_COUNT, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPES, which isn't translated (it uses Minecraft's `RenderType`))).
        // EMISSIVE_MASK: not translated, it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))))).
        // DIFFUSE_MASK: not translated, it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))))))).
        // AO_MASK: not translated, it uses EncodingFormat.AO_BIT_LENGTH, which isn't translated (it uses EncodingFormat.TRI_STATE_COUNT, which isn't translated (it uses EncodingFormat.TRI_STATES, which isn't translated (it uses Fabric's `TriState`))).
        // FOIL_TYPE_MASK: not translated, it uses EncodingFormat.FOIL_TYPE_BIT_LENGTH, which isn't translated (it uses EncodingFormat.NULLABLE_FOIL_TYPE_COUNT, which isn't translated (it uses EncodingFormat.NULLABLE_FOIL_TYPES, which isn't translated (it uses Minecraft's `FoilType`))).
        // SHADE_MODE_MASK: not translated, it uses EncodingFormat.SHADE_MODE_BIT_LENGTH, which isn't translated (it uses EncodingFormat.SHADE_MODE_COUNT, which isn't translated (it uses EncodingFormat.SHADE_MODES, which isn't translated (it uses Fabric's `ShadeMode`))).
        // ANIMATED_MASK: not translated, it uses EncodingFormat.ANIMATED_BIT_OFFSET, which isn't translated (it uses EncodingFormat.SHADE_MODE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.FOIL_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.AO_BIT_OFFSET, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))))))))))).
        #[derive(Clone, Debug)]
        pub struct EncodingFormat {
        }
        impl EncodingFormat {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn bit_mask(bit_length: i32, bit_offset: i32) -> i32 {
            return ((1 << bit_length) - 1) << bit_offset;
        }
        // EncodingFormat.cullFace(1): not translated, it uses EncodingFormat.CULL_MASK, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)).
        // EncodingFormat.cullFace(2): not translated, it uses EncodingFormat.CULL_MASK, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)).
        // EncodingFormat.lightFace(1): not translated, it uses EncodingFormat.LIGHT_MASK, which isn't translated (it uses EncodingFormat.LIGHT_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)).
        // EncodingFormat.lightFace(2): not translated, it uses EncodingFormat.LIGHT_MASK, which isn't translated (it uses EncodingFormat.LIGHT_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)).
        // EncodingFormat.normalFlags(1): not translated, it uses EncodingFormat.NORMALS_MASK, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))).
        // EncodingFormat.normalFlags(2): not translated, it uses EncodingFormat.NORMALS_MASK, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))).
        // EncodingFormat.geometryFlags(1): not translated, it uses EncodingFormat.GEOMETRY_MASK, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))).
        // EncodingFormat.geometryFlags(2): not translated, it uses EncodingFormat.GEOMETRY_MASK, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))).
        // EncodingFormat.quadAtlas(1): not translated, it uses Fabric's `QuadAtlas`.
        // EncodingFormat.quadAtlas(2): not translated, it uses Fabric's `QuadAtlas`.
        // EncodingFormat.chunkLayer(1): not translated, it uses Minecraft's `ChunkSectionLayer`.
        // EncodingFormat.chunkLayer(2): not translated, it uses Minecraft's `ChunkSectionLayer`.
        // EncodingFormat.itemRenderType(1): not translated, it uses Minecraft's `RenderType`.
        // EncodingFormat.itemRenderType(2): not translated, it uses ItemRenderType, which isn't translated (its constants carry values).
        // EncodingFormat.emissive(1): not translated, it uses EncodingFormat.EMISSIVE_MASK, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))))))).
        // EncodingFormat.emissive(2): not translated, it uses EncodingFormat.EMISSIVE_MASK, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2))))))))).
        // EncodingFormat.diffuseShade(1): not translated, it uses EncodingFormat.DIFFUSE_MASK, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))))))).
        // EncodingFormat.diffuseShade(2): not translated, it uses EncodingFormat.DIFFUSE_MASK, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))))))).
        // EncodingFormat.ambientOcclusion(1): not translated, it uses Fabric's `TriState`.
        // EncodingFormat.ambientOcclusion(2): not translated, it uses Fabric's `TriState`.
        // EncodingFormat.foilType(1): not translated, it uses Minecraft's `FoilType`.
        // EncodingFormat.foilType(2): not translated, it uses Minecraft's `FoilType`.
        // EncodingFormat.shadeMode(1): not translated, it uses Fabric's `ShadeMode`.
        // EncodingFormat.shadeMode(2): not translated, it uses Fabric's `ShadeMode`.
        // EncodingFormat.animated(1): not translated, it uses EncodingFormat.ANIMATED_MASK, which isn't translated (it uses EncodingFormat.ANIMATED_BIT_OFFSET, which isn't translated (it uses EncodingFormat.SHADE_MODE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.FOIL_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.AO_BIT_OFFSET, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))))))))))).
        // EncodingFormat.animated(2): not translated, it uses EncodingFormat.ANIMATED_MASK, which isn't translated (it uses EncodingFormat.ANIMATED_BIT_OFFSET, which isn't translated (it uses EncodingFormat.SHADE_MODE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.FOIL_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.AO_BIT_OFFSET, which isn't translated (it uses EncodingFormat.DIFFUSE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.EMISSIVE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.ITEM_RENDER_TYPE_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CHUNK_LAYER_BIT_OFFSET, which isn't translated (it uses EncodingFormat.QUAD_ATLAS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.GEOMETRY_BIT_OFFSET, which isn't translated (it uses EncodingFormat.NORMALS_BIT_OFFSET, which isn't translated (it uses EncodingFormat.LIGHT_BIT_OFFSET, which isn't translated (it uses EncodingFormat.CULL_BIT_LENGTH, which isn't translated (it calls Minecraft's `Mth`.ceillog2)))))))))))))).
    }

    /// `ItemRenderType` (net/vulkanmod/render/chunk/build/frapi/mesh/ItemRenderType.java)
    pub mod item_render_type {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // RENDER_TYPES: not translated, it uses Minecraft's `RenderType`.
        // RENDER_TYPE_2_ENUM: not translated, it uses Minecraft's `RenderType`.
        // DEFAULT: not translated, it uses ItemRenderType, which isn't translated (its constants carry values).
        // ItemRenderType.ItemRenderType(1): not translated, its class isn't translated (its constants carry values).
    }

    /// `MeshImpl` (net/vulkanmod/render/chunk/build/frapi/mesh/MeshImpl.java)
    pub mod mesh_impl {
        use super::*;
        // The class itself isn't translated: it extends `MeshViewImpl`. Its static parts are below.
        // MeshImpl.MeshImpl(1): not translated, its class isn't translated (it extends `MeshViewImpl`).
    }

    /// `MeshViewImpl` (net/vulkanmod/render/chunk/build/frapi/mesh/MeshViewImpl.java)
    pub mod mesh_view_impl {
        use super::*;
        // The class itself isn't translated: it implements Fabric's `MeshView`. Its static parts are below.
        // CURSOR_POOLS: not translated, it uses `ThreadLocal`.
        // MeshViewImpl.MeshViewImpl(0): not translated, its class isn't translated (it implements Fabric's `MeshView`).
        // MeshViewImpl.size(0): not translated, its class isn't translated (it implements Fabric's `MeshView`).
        // MeshViewImpl.forEach(1): not translated, its class isn't translated (it implements Fabric's `MeshView`).
        // MeshViewImpl.forEach(2): not translated, it's generic.
        // MeshViewImpl.outputTo(1): not translated, its class isn't translated (it implements Fabric's `MeshView`).
    }

    /// `MutableMeshImpl` (net/vulkanmod/render/chunk/build/frapi/mesh/MutableMeshImpl.java)
    pub mod mutable_mesh_impl {
        use super::*;
        // The class itself isn't translated: it extends `MeshViewImpl`. Its static parts are below.
        // MutableMeshImpl.MutableMeshImpl(0): not translated, its class isn't translated (it extends `MeshViewImpl`).
        // MutableMeshImpl.ensureCapacity(1): not translated, its class isn't translated (it extends `MeshViewImpl`).
        // MutableMeshImpl.emitter(0): not translated, its class isn't translated (it extends `MeshViewImpl`).
        // MutableMeshImpl.forEachMutable(1): not translated, its class isn't translated (it extends `MeshViewImpl`).
        // MutableMeshImpl.immutableCopy(0): not translated, its class isn't translated (it extends `MeshViewImpl`).
        // MutableMeshImpl.clear(0): not translated, its class isn't translated (it extends `MeshViewImpl`).
    }

    /// `MutableQuadViewImpl` (net/vulkanmod/render/chunk/build/frapi/mesh/MutableQuadViewImpl.java)
    pub mod mutable_quad_view_impl {
        use super::*;
        // The class itself isn't translated: it extends `QuadViewImpl`. Its static parts are below.
        // NO_TRANSFORM: not translated, it uses Fabric's `QuadTransform`.
        pub static DEFAULT_QUAD_DATA: LazyLock<Vec<i32>> = LazyLock::new(|| vec![0i32; encoding_format::TOTAL_STRIDE as usize]);
        // MutableQuadViewImpl.pos(4): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.translate(3): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.color(2): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.uv(3): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.lightmap(2): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.normalFlags(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.normal(4): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.populateMissingNormals(0): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.nominalFace(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.cullFace(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.chunkLayer(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.itemRenderType(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.emissive(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.diffuseShade(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.ambientOcclusion(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.foilType(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.shadeMode(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.animated(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.atlas(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.tintIndex(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.tag(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.copyFrom(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.fromBakedQuad(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.clear(0): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.pushTransform(1): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.popTransform(0): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.emitDirectly(0): not translated, it has no body.
        // MutableQuadViewImpl.transformAndEmit(0): not translated, its class isn't translated (it extends `QuadViewImpl`).
        // MutableQuadViewImpl.emit(0): not translated, its class isn't translated (it extends `QuadViewImpl`).
    }

    /// `QuadViewImpl` (net/vulkanmod/render/chunk/build/frapi/mesh/QuadViewImpl.java)
    pub mod quad_view_impl {
        use super::*;
        // The class itself isn't translated: it implements Fabric's `QuadView`. Its static parts are below.
        // QuadViewImpl.load(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.computeGeometry(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.geometryFlags(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.x(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.y(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.z(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.posByIndex(2): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.copyPos(2): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.color(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.u(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.v(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.copyUv(2): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.lightmap(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.normalFlags(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.hasNormal(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.hasVertexNormals(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.hasAllVertexNormals(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.normalIndex(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.normalX(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.normalY(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.normalZ(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.copyNormal(2): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.packedFaceNormal(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.faceNormal(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.lightFace(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.nominalFace(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.cullFace(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.atlas(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.chunkLayer(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.itemRenderType(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.emissive(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.diffuseShade(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.ambientOcclusion(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.foilType(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.shadeMode(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.animated(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.tintIndex(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.tag(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.buffer(2): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.buffer(3): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getFlags(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getX(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getY(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getZ(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getColor(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getU(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getV(1): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getColorIndex(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getFacingDirection(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getNormal(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
        // QuadViewImpl.getQuadFacing(0): not translated, its class isn't translated (it implements Fabric's `QuadView`).
    }

    /// `AltItemRenderer` (net/vulkanmod/render/chunk/build/frapi/render/AltItemRenderer.java)
    pub mod alt_item_renderer {
        use super::*;
        // The class itself isn't translated: field emitter: an anonymous class. Its static parts are below.
        // AltItemRenderer.prepare(3): not translated, its class isn't translated (field emitter: an anonymous class).
        // AltItemRenderer.clear(0): not translated, its class isn't translated (field emitter: an anonymous class).
        // AltItemRenderer.renderItem(1): not translated, its class isn't translated (field emitter: an anonymous class).
        // AltItemRenderer.bufferQuads(2): not translated, its class isn't translated (field emitter: an anonymous class).
        // AltItemRenderer.bufferQuad(1): not translated, its class isn't translated (field emitter: an anonymous class).
        // AltItemRenderer.shadeQuad(2): not translated, its class isn't translated (field emitter: an anonymous class).
        // AltItemRenderer.tintQuad(1): not translated, its class isn't translated (field emitter: an anonymous class).
    }

    /// `AltModelBlockRendererImpl` (net/vulkanmod/render/chunk/build/frapi/render/AltModelBlockRendererImpl.java)
    pub mod alt_model_block_renderer_impl {
        use super::*;
        // The class itself isn't translated: it extends `AbstractBlockRenderContext` (net.vulkanmod.render.chunk.build.renderer). Its static parts are below.
        // AltModelBlockRendererImpl.AltModelBlockRendererImpl(3): not translated, its class isn't translated (it extends `AbstractBlockRenderContext` (net.vulkanmod.render.chunk.build.renderer)).
        // AltModelBlockRendererImpl.setup(2): not translated, its class isn't translated (it extends `AbstractBlockRenderContext` (net.vulkanmod.render.chunk.build.renderer)).
        // AltModelBlockRendererImpl.tesselateBlock(9): not translated, its class isn't translated (it extends `AbstractBlockRenderContext` (net.vulkanmod.render.chunk.build.renderer)).
        // AltModelBlockRendererImpl.transform(1): not translated, its class isn't translated (it extends `AbstractBlockRenderContext` (net.vulkanmod.render.chunk.build.renderer)).
    }

    /// `VulkanModRenderer` (net/vulkanmod/render/chunk/build/frapi/VulkanModRenderer.java)
    pub mod vulkan_mod_renderer {
        use super::*;
        // The class itself isn't translated: it implements Fabric's `Renderer`. Its static parts are below.
        // INSTANCE: not translated, it uses VulkanModRenderer, which isn't translated (it implements Fabric's `Renderer`).
        // modelBlockRenderer: not translated, it uses AltModelBlockRendererImpl, which isn't translated (it extends `AbstractBlockRenderContext` (net.vulkanmod.render.chunk.build.renderer)).
        // VulkanModRenderer.VulkanModRenderer(0): not translated, its class isn't translated (it implements Fabric's `Renderer`).
        // VulkanModRenderer.getOrCreateInstance(0): not translated, it uses VulkanModRenderer, which isn't translated (it implements Fabric's `Renderer`).
        // VulkanModRenderer.quadEmitter(1): not translated, its body couldn't be read: an anonymous class.
        // VulkanModRenderer.mutableMesh(0): not translated, its class isn't translated (it implements Fabric's `Renderer`).
        // VulkanModRenderer.altModelBlockRenderer(3): not translated, its class isn't translated (it implements Fabric's `Renderer`).
    }

    /// `ArrayLightDataCache` (net/vulkanmod/render/chunk/build/light/data/ArrayLightDataCache.java)
    pub mod array_light_data_cache {
        use super::*;
        // The class itself isn't translated: it extends `LightDataAccess`. Its static parts are below.
        pub const NEIGHBOR_BLOCK_RADIUS: i32 = 2;
        pub const BLOCK_LENGTH: i32 = 20;
        // ArrayLightDataCache.reset(4): not translated, its class isn't translated (it extends `LightDataAccess`).
        // ArrayLightDataCache.reset(2): not translated, its class isn't translated (it extends `LightDataAccess`).
        // ArrayLightDataCache.reset(1): not translated, its class isn't translated (it extends `LightDataAccess`).
        // ArrayLightDataCache.reset(1): not translated, its class isn't translated (it extends `LightDataAccess`).
        // ArrayLightDataCache.index(3): not translated, its class isn't translated (it extends `LightDataAccess`).
        // ArrayLightDataCache.get(3): not translated, its class isn't translated (it extends `LightDataAccess`).
    }

    /// `LightDataAccess` (net/vulkanmod/render/chunk/build/light/data/LightDataAccess.java)
    pub mod light_data_access {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `MutableBlockPos`. Its static parts are below.
        pub const BL_OFFSET: i32 = 0;
        pub const SL_OFFSET: i32 = 4;
        pub const AO_OFFSET: i32 = 8;
        pub const CO_OFFSET: i32 = 20;
        pub const EM_OFFSET: i32 = 28;
        pub const OP_OFFSET: i32 = 29;
        pub const FO_OFFSET: i32 = 30;
        pub const FC_OFFSET: i32 = 31;
        pub const AO_INV: f32 = 4.8828125e-4f32;
        // LightDataAccess.LightDataAccess(0): not translated, its class isn't translated (it uses Minecraft's `MutableBlockPos`).
        // LightDataAccess.get(5): not translated, its class isn't translated (it uses Minecraft's `MutableBlockPos`).
        // LightDataAccess.get(4): not translated, its class isn't translated (it uses Minecraft's `MutableBlockPos`).
        // LightDataAccess.get(2): not translated, its class isn't translated (it uses Minecraft's `MutableBlockPos`).
        // LightDataAccess.get(1): not translated, its class isn't translated (it uses Minecraft's `MutableBlockPos`).
        // LightDataAccess.get(3): not translated, it has no body.
        // LightDataAccess.compute(3): not translated, its class isn't translated (it uses Minecraft's `MutableBlockPos`).
        pub fn pack_bl(block_light: i32) -> i32 {
            return (block_light & 15) << 0;
        }
        pub fn unpack_bl(word: i32) -> i32 {
            return (((word as u32) >> 0) as i32) & 15;
        }
        pub fn pack_sl(sky_light: i32) -> i32 {
            return (sky_light & 15) << 4;
        }
        pub fn unpack_sl(word: i32) -> i32 {
            return (((word as u32) >> 4) as i32) & 15;
        }
        pub fn pack_ao(ao: f32) -> i32 {
            let aoi: i32 = ((ao * 2048.0f32) as i32);
            return (aoi & 4095) << 8;
        }
        pub fn unpack_ao(word: i32) -> f32 {
            let aoi: i32 = (((word as u32) >> 8) as i32) & 4095;
            return (aoi as f32) * (4.8828125e-4f32);
        }
        pub fn pack_co(co: i32) -> i32 {
            return (co & 0xFF) << 20;
        }
        pub fn unpack_co(word: i32) -> i32 {
            return (((word as u32) >> 20) as i32) & 0xFF;
        }
        pub fn pack_em(emissive: bool) -> i32 {
            return (if emissive { 1 } else { 0 }) << 28;
        }
        pub fn unpack_em(word: i32) -> bool {
            return ((((word as u32) >> 28) as i32) & 1) != 0;
        }
        pub fn pack_op(opaque: bool) -> i32 {
            return (if opaque { 1 } else { 0 }) << 29;
        }
        pub fn unpack_op(word: i32) -> bool {
            return ((((word as u32) >> 29) as i32) & 1) != 0;
        }
        pub fn pack_fo(opaque: bool) -> i32 {
            return (if opaque { 1 } else { 0 }) << 30;
        }
        pub fn unpack_fo(word: i32) -> bool {
            return ((((word as u32) >> 30) as i32) & 1) != 0;
        }
        pub fn pack_fc(full_cube: bool) -> i32 {
            return (if full_cube { 1 } else { 0 }) << 31;
        }
        pub fn unpack_fc(word: i32) -> bool {
            return ((((word as u32) >> 31) as i32) & 1) != 0;
        }
        // LightDataAccess.getLightmap(1): not translated, it calls Minecraft's `LightCoordsUtil`.pack.
        // LightDataAccess.getEmissiveLightmap(1): not translated, it calls LightDataAccess.getLightmap, which isn't translated.
        // LightDataAccess.getRegion(0): not translated, its class isn't translated (it uses Minecraft's `MutableBlockPos`).
    }

    /// `QuadLightData` (net/vulkanmod/render/chunk/build/light/data/QuadLightData.java)
    pub mod quad_light_data {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct QuadLightData {
            pub br: Vec<f32>,
            pub lm: Vec<i32>,
        }
        impl QuadLightData {
            pub fn new() -> Self {
                let mut s = Self { br: vec![0.0f32; 4], lm: vec![0i32; 4] };
                s
            }
        }
    }

    /// `FlatLightPipeline` (net/vulkanmod/render/chunk/build/light/flat/FlatLightPipeline.java)
    pub mod flat_light_pipeline {
        use super::*;
        // The class itself isn't translated: it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light). Its static parts are below.
        // FlatLightPipeline.FlatLightPipeline(1): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // FlatLightPipeline.calculate(6): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // FlatLightPipeline.getLightmap(2): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
    }

    /// `LightMode` (net/vulkanmod/render/chunk/build/light/LightMode.java)
    pub mod light_mode {
        use super::*;
        pub const FLAT: i32 = 0;
        pub const SMOOTH: i32 = 1;
        pub const SUB_BLOCK: i32 = 2;
        #[derive(Clone, Debug)]
        pub struct LightMode {
        }
        impl LightMode {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
    }

    /// `LightPipeline` (net/vulkanmod/render/chunk/build/light/LightPipeline.java)
    pub mod light_pipeline {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // LightPipeline.calculate(6): not translated, it has no body.
    }

    /// `AoFaceData` (net/vulkanmod/render/chunk/build/light/smooth/AoFaceData.java)
    pub mod ao_face_data {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct AoFaceData {
            pub lm: Vec<i32>,
            pub ao: Vec<f32>,
            pub bl: Vec<f32>,
            pub sl: Vec<f32>,
            pub flags: i32,
        }
        impl AoFaceData {
            pub fn new() -> Self {
                let mut s = Self { lm: vec![0i32; 4], ao: vec![0.0f32; 4], bl: vec![0.0f32; 4], sl: vec![0.0f32; 4], flags: 0i32 };
                s
            }
            // AoFaceData.initLightData(4): not translated, it uses LightDataAccess, which isn't translated (it uses Minecraft's `MutableBlockPos`).
            // AoFaceData.unpackLightData(0): not translated, it changes an array or object through a second name (bl).
            pub fn get_corner_occlusion(&self, bits: i32, values: &[i32], i: i32) -> bool {
                return (bits & (1 << values[i as usize])) != 0;
            }
            pub fn get_blended_sky_light(&self, w: &[f32]) -> f32 {
                return ao_face_data::weighted_sum(&self.sl, w);
            }
            pub fn get_blended_block_light(&self, w: &[f32]) -> f32 {
                return ao_face_data::weighted_sum(&self.bl, w);
            }
            pub fn get_blended_shade(&self, w: &[f32]) -> f32 {
                return ao_face_data::weighted_sum(&self.ao, w);
            }
            pub fn has_light_data(&self) -> bool {
                return (self.flags & 1) != 0;
            }
            pub fn has_unpacked_light_data(&self) -> bool {
                return (self.flags & 2) != 0;
            }
            pub fn reset(&mut self) {
                self.flags = 0;
            }
        }
        pub fn weighted_sum(v: &[f32], w: &[f32]) -> f32 {
            let t0: f32 = v[0] * w[0];
            let t1: f32 = v[1] * w[1];
            let t2: f32 = v[2] * w[2];
            let t3: f32 = v[3] * w[3];
            return ((t0 + t1) + t2) + t3;
        }
        pub fn unpack_sky_light(i: i32) -> f32 {
            return (((i >> 16) & 0xFF) as f32);
        }
        pub fn unpack_block_light(i: i32) -> f32 {
            return ((i & 0xFF) as f32);
        }
        pub fn calculate_corner_brightness(mut a: i32, mut b: i32, mut c: i32, mut d: i32, aem: bool, bem: bool, cem: bool, dem: bool) -> i32 {
            if (((a == 0) || (b == 0)) || (c == 0)) || (d == 0) {
                let min: i32 = ao_face_data::min_non_zero(ao_face_data::min_non_zero(a, b), ao_face_data::min_non_zero(c, d));
                a = a.max(min);
                b = b.max(min);
                c = c.max(min);
                d = d.max(min);
            }
            if aem {
                a = 15728880;
            }
            if bem {
                b = 15728880;
            }
            if cem {
                c = 15728880;
            }
            if dem {
                d = 15728880;
            }
            return ((((a + b) + c) + d) >> 2) & 16711935;
        }
        pub fn min_non_zero(a: i32, b: i32) -> i32 {
            if a == 0 {
                return b;
            } else {
                return if b == 0 { a } else { a.min(b) };
            }
            #[allow(unreachable_code)]
            unreachable!()
        }
    }

    /// `AoNeighborInfo` (net/vulkanmod/render/chunk/build/light/smooth/AoNeighborInfo.java)
    pub mod ao_neighbor_info {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // VALUES: not translated, it uses AoNeighborInfo, which isn't translated (its constants carry values).
        // AoNeighborInfo.AoNeighborInfo(3): not translated, its class isn't translated (its constants carry values).
        // AoNeighborInfo.calculateCornerWeights(4): not translated, it has no body.
        // AoNeighborInfo.getU(3): not translated, it has no body.
        // AoNeighborInfo.getV(3): not translated, it has no body.
        // AoNeighborInfo.copyLightValues(4): not translated, its class isn't translated (its constants carry values).
        // AoNeighborInfo.getDepth(3): not translated, it has no body.
        // AoNeighborInfo.get(1): not translated, it uses AoNeighborInfo, which isn't translated (its constants carry values).
        pub fn calculate_corner_weights_3(u: f32, v: f32, out: &mut [f32]) {
            out[0] = u * v;
            out[1] = u * (1.0f32 - v);
            out[2] = (1.0f32 - u) * (1.0f32 - v);
            out[3] = (1.0f32 - u) * v;
        }
        pub fn copy_in_corner_bits(corners_bits: &mut [i32], idxs: &[i32]) {
            corners_bits[0] = idxs[0];
            corners_bits[1] = idxs[1];
            corners_bits[2] = idxs[2];
            corners_bits[3] = idxs[3];
            corners_bits[4] = idxs[4];
            corners_bits[5] = idxs[5];
            corners_bits[6] = idxs[6];
            corners_bits[7] = idxs[7];
        }
        pub fn get_out_corner_bits(corners_bits: &mut [i32], idxs: &[i32]) {
            corners_bits[0] = idxs[0];
            corners_bits[1] = idxs[3];
            corners_bits[2] = idxs[1];
            corners_bits[3] = idxs[1];
            corners_bits[4] = idxs[2];
            corners_bits[5] = idxs[3];
            corners_bits[6] = idxs[3];
            corners_bits[7] = idxs[0];
            corners_bits[8] = idxs[2];
            corners_bits[9] = idxs[2];
            corners_bits[10] = idxs[1];
            corners_bits[11] = idxs[0];
            corners_bits[12] = idxs[4];
            corners_bits[13] = idxs[7];
            corners_bits[14] = idxs[5];
            corners_bits[15] = idxs[5];
            corners_bits[16] = idxs[6];
            corners_bits[17] = idxs[7];
            corners_bits[18] = idxs[7];
            corners_bits[19] = idxs[4];
            corners_bits[20] = idxs[6];
            corners_bits[21] = idxs[6];
            corners_bits[22] = idxs[5];
            corners_bits[23] = idxs[4];
        }
    }

    /// `FaceDataFlags` (net/vulkanmod/render/chunk/build/light/smooth/FaceDataFlags.java)
    pub mod face_data_flags {
        use super::*;
        pub const HAS_LIGHT_DATA: i32 = 1;
        pub const HAS_UNPACKED_LIGHT_DATA: i32 = 2;
        #[derive(Clone, Debug)]
        pub struct FaceDataFlags {
        }
        impl FaceDataFlags {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
    }

    /// `NewSmoothLightPipeline` (net/vulkanmod/render/chunk/build/light/smooth/NewSmoothLightPipeline.java)
    pub mod new_smooth_light_pipeline {
        use super::*;
        // The class itself isn't translated: it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light). Its static parts are below.
        // NewSmoothLightPipeline.NewSmoothLightPipeline(1): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.calculate(6): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.applyAlignedFullFace(4): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.applyAlignedPartialFace(5): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.applyParallelFace(5): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.applyNonParallelFace(5): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.applyAlignedPartialFaceVertex(6): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.applyInsetPartialFaceVertex(7): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.applyInsetPartialFaceVertexSO(7): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.applySidedBrightness(3): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.getCachedFaceData(3): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // NewSmoothLightPipeline.updateCachedData(1): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        pub fn clamp(v: f32) -> f32 {
            if v < 0.0f32 {
                return 0.0f32;
            } else {
                return if v > 1.0f32 { 1.0f32 } else { v };
            }
            #[allow(unreachable_code)]
            unreachable!()
        }
        pub fn pack_light_map(sl: f32, bl: f32) -> i32 {
            return (((sl as i32) & 0xFF) << 16) | ((bl as i32) & 0xFF);
        }
    }

    /// `SmoothLightPipeline` (net/vulkanmod/render/chunk/build/light/smooth/SmoothLightPipeline.java)
    pub mod smooth_light_pipeline {
        use super::*;
        // The class itself isn't translated: it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light). Its static parts are below.
        // SmoothLightPipeline.SmoothLightPipeline(1): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.calculate(6): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.applyAlignedFullFace(4): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.applyAlignedPartialFace(5): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.applyParallelFace(5): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.applyNonParallelFace(5): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.applyAlignedPartialFaceVertex(6): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.applyInsetPartialFaceVertex(7): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.applySidedBrightness(3): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.getCachedFaceData(3): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        // SmoothLightPipeline.updateCachedData(1): not translated, its class isn't translated (it implements `LightPipeline` (net.vulkanmod.render.chunk.build.light)).
        pub fn clamp(v: f32) -> f32 {
            if v < 0.0f32 {
                return 0.0f32;
            } else {
                return if v > 1.0f32 { 1.0f32 } else { v };
            }
            #[allow(unreachable_code)]
            unreachable!()
        }
        pub fn pack_light_map(sl: f32, bl: f32) -> i32 {
            return (((sl as i32) & 0xFF) << 16) | ((bl as i32) & 0xFF);
        }
    }

    /// `SubBlockAoFace` (net/vulkanmod/render/chunk/build/light/smooth/SubBlockAoFace.java)
    pub mod sub_block_ao_face {
        use super::*;
        // The class itself isn't translated: it extends `AoFaceData`. Its static parts are below.
        // SubBlockAoFace.initLightData(4): not translated, its class isn't translated (it extends `AoFaceData`).
        // SubBlockAoFace.calculateSelfOcclusion(3): not translated, its class isn't translated (it extends `AoFaceData`).
        // SubBlockAoFace.calculatePartialAlignedFace(3): not translated, its class isn't translated (it extends `AoFaceData`).
    }

    /// `AbstractBlockRenderContext` (net/vulkanmod/render/chunk/build/renderer/AbstractBlockRenderContext.java)
    pub mod abstract_block_render_context {
        use super::*;
        // The class itself isn't translated: it extends `AbstractRenderContext`. Its static parts are below.
        // AbstractBlockRenderContext.AbstractBlockRenderContext(0): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.setupLightPipelines(2): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.prepareForWorld(2): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.prepareForBlock(3): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.isFaceCulled(1): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.shouldRenderFace(1): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.faceNotOccluded(2): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.getEmitter(0): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.bufferQuad(1): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.renderQuad(1): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.endRenderQuad(1): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.tintQuad(1): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.configureTintCache(1): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.computeTintColor(4): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.getTintColor(4): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.resetTintCache(0): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.shadeQuad(4): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.effectiveRenderLayer(1): not translated, its class isn't translated (it extends `AbstractRenderContext`).
        // AbstractBlockRenderContext.emitVanillaBlockQuads(2): not translated, its class isn't translated (it extends `AbstractRenderContext`).
    }

    /// `AbstractBlockRenderContext.ShapePairKey` (net/vulkanmod/render/chunk/build/renderer/AbstractBlockRenderContext.java)
    pub mod abstract_block_render_context_shape_pair_key {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `VoxelShape`. Its static parts are below.
        // AbstractBlockRenderContext.ShapePairKey.ShapePairKey(2): not translated, its class isn't translated (it uses Minecraft's `VoxelShape`).
        // AbstractBlockRenderContext.ShapePairKey.equals(1): not translated, its class isn't translated (it uses Minecraft's `VoxelShape`).
        // AbstractBlockRenderContext.ShapePairKey.hashCode(0): not translated, its class isn't translated (it uses Minecraft's `VoxelShape`).
        // AbstractBlockRenderContext.ShapePairKey.first(0): not translated, its class isn't translated (it uses Minecraft's `VoxelShape`).
        // AbstractBlockRenderContext.ShapePairKey.second(0): not translated, its class isn't translated (it uses Minecraft's `VoxelShape`).
    }

    /// `AbstractRenderContext` (net/vulkanmod/render/chunk/build/renderer/AbstractRenderContext.java)
    pub mod abstract_render_context {
        use super::*;
        // The class itself isn't translated: field editorQuad: an anonymous class. Its static parts are below.
        // AbstractRenderContext.bufferQuad(1): not translated, it has no body.
        // AbstractRenderContext.bufferQuad(2): not translated, its class isn't translated (field editorQuad: an anonymous class).
    }

    /// `BlockRenderer` (net/vulkanmod/render/chunk/build/renderer/BlockRenderer.java)
    pub mod block_renderer {
        use super::*;
        // The class itself isn't translated: it extends `AbstractBlockRenderContext`. Its static parts are below.
        // BlockRenderer.setResources(1): not translated, its class isn't translated (it extends `AbstractBlockRenderContext`).
        // BlockRenderer.BlockRenderer(2): not translated, its class isn't translated (it extends `AbstractBlockRenderContext`).
        // BlockRenderer.renderBlock(3): not translated, its class isn't translated (it extends `AbstractBlockRenderContext`).
        // BlockRenderer.endRenderQuad(1): not translated, its class isn't translated (it extends `AbstractBlockRenderContext`).
        // BlockRenderer.getBufferBuilder(1): not translated, its class isn't translated (it extends `AbstractBlockRenderContext`).
        // BlockRenderer.bufferQuad(4): not translated, its class isn't translated (it extends `AbstractBlockRenderContext`).
    }

    /// `DefaultFluidRenderers` (net/vulkanmod/render/chunk/build/renderer/DefaultFluidRenderers.java)
    pub mod default_fluid_renderers {
        use super::*;
        // set: not translated, it uses `ReferenceOpenHashSet` (it.unimi.dsi.fastutil.objects).
        #[derive(Clone, Debug)]
        pub struct DefaultFluidRenderers {
        }
        impl DefaultFluidRenderers {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // DefaultFluidRenderers.add(1): not translated, it uses Fabric's `FluidRenderHandler`.
        // DefaultFluidRenderers.has(1): not translated, it uses Fabric's `FluidRenderHandler`.
    }

    /// `FluidRenderer` (net/vulkanmod/render/chunk/build/renderer/FluidRenderer.java)
    pub mod fluid_renderer {
        use super::*;
        // The class itself isn't translated: it implements Fabric's `DefaultRenderer`. Its static parts are below.
        pub const MAX_FLUID_HEIGHT: f32 = 0.8888889f32;
        // FluidRenderer.FluidRenderer(2): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.setResources(1): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.renderLiquid(3): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.isFaceOccludedByState(5): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.shouldRenderFace(6): not translated, it uses Minecraft's `BlockAndTintGetter`.
        // FluidRenderer.getAdjBlockState(5): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.render(7): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.render(3): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.calculateAverageHeight(6): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.addWeightedHeight(2): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.getHeight(3): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.getHeight(4): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.calculateNormal(1): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.putQuad(6): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.setVertex(7): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.updateQuad(4): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
        // FluidRenderer.updateColor(4): not translated, its class isn't translated (it implements Fabric's `DefaultRenderer`).
    }

    /// `RenderRegion` (net/vulkanmod/render/chunk/build/RenderRegion.java)
    pub mod render_region {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `BlockAndTintGetter`. Its static parts are below.
        pub const WIDTH: i32 = 3;
        pub const SIZE: i32 = 27;
        pub const BOUNDARY_BLOCK_WIDTH: i32 = 2;
        pub const REGION_BLOCK_WIDTH: i32 = 20;
        pub const BLOCK_COUNT: i32 = 8000;
        // AIR_BLOCK_STATE: not translated, it uses Minecraft's `BlockState`.
        // RenderRegion.RenderRegion(8): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.loadBlockStates(0): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.loadSectionBlockStates(8): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.initTintCache(1): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getBlockState(1): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getFluidState(1): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getLightEngine(0): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getBrightness(2): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getRawBrightness(2): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getBlockEntity(1): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.cardinalLighting(0): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getBlockTint(2): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getMinY(0): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getHeight(0): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getSectionIdx(3): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.getBlockIdx(3): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.outsideRegion(3): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.defaultBlockState(1): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
        // RenderRegion.debugBlockState(1): not translated, its class isn't translated (it implements Minecraft's `BlockAndTintGetter`).
    }

    /// `RenderRegionBuilder` (net/vulkanmod/render/chunk/build/RenderRegionBuilder.java)
    pub mod render_region_builder {
        use super::*;
        // The class itself isn't translated: it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs). Its static parts are below.
        // DEFAULT_SKY_LIGHT_DATA_LAYER: not translated, it uses Minecraft's `DataLayer`.
        // DEFAULT_BLOCK_LIGHT_DATA_LAYER: not translated, it uses Minecraft's `DataLayer`.
        pub const MAX_CACHE_ENTRIES: i32 = 256;
        // RenderRegionBuilder.createRegion(4): not translated, its class isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // RenderRegionBuilder.getSectionDataLayers(2): not translated, its class isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // RenderRegionBuilder.getLevelChunk(3): not translated, its class isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // RenderRegionBuilder.remove(2): not translated, its class isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // RenderRegionBuilder.clear(0): not translated, its class isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs)).
    }

    /// `BuildTask` (net/vulkanmod/render/chunk/build/task/BuildTask.java)
    pub mod build_task {
        use super::*;
        // The class itself isn't translated: it extends `ChunkTask`. Its static parts are below.
        // BuildTask.BuildTask(4): not translated, its class isn't translated (it extends `ChunkTask`).
        // BuildTask.name(0): not translated, its class isn't translated (it extends `ChunkTask`).
        // BuildTask.runTask(1): not translated, its class isn't translated (it extends `ChunkTask`).
        // BuildTask.compile(4): not translated, its class isn't translated (it extends `ChunkTask`).
        // BuildTask.setupBufferBuilders(1): not translated, its class isn't translated (it extends `ChunkTask`).
        // BuildTask.getTerrainBuilder(2): not translated, its class isn't translated (it extends `ChunkTask`).
        // BuildTask.compactRenderTypes(1): not translated, its class isn't translated (it extends `ChunkTask`).
        // BuildTask.handleBlockEntity(2): not translated, it's generic.
    }

    /// `ChunkTask` (net/vulkanmod/render/chunk/build/task/ChunkTask.java)
    pub mod chunk_task {
        use super::*;
        // The class itself isn't translated: it keeps an atomic value in an object's field. Its static parts are below.
        pub const BENCH: bool = true;
        // taskDispatcher: not translated, it uses TaskDispatcher, which isn't translated (it uses `Thread`).
        // ChunkTask.createBuildTask(4): not translated, it uses BuildTask, which isn't translated (it extends `ChunkTask`).
        // ChunkTask.createSortTask(2): not translated, it uses SortTransparencyTask, which isn't translated (it extends `ChunkTask`).
        // ChunkTask.ChunkTask(2): not translated, its class isn't translated (it keeps an atomic value in an object's field).
        // ChunkTask.name(0): not translated, it has no body.
        // ChunkTask.runTask(1): not translated, it has no body.
        // ChunkTask.cancel(0): not translated, its class isn't translated (it keeps an atomic value in an object's field).
        // ChunkTask.setTaskDispatcher(1): not translated, it stores a second reference to an object (Rust would need it copied or shared).
    }

    /// `ChunkTask.Result` (net/vulkanmod/render/chunk/build/task/ChunkTask.java)
    pub mod chunk_task_result {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum Result_ {
            Cancelled,
            Successful,
        }
        impl Result_ {
            pub const VALUES: [Result_; 2] = [Result_::Cancelled, Result_::Successful, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    Result_::Cancelled => "CANCELLED",
                    Result_::Successful => "SUCCESSFUL",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "CANCELLED" => Result_::Cancelled,
                    "SUCCESSFUL" => Result_::Successful,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `CompiledSection` (net/vulkanmod/render/chunk/build/task/CompiledSection.java)
    pub mod compiled_section {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `BlockEntity`. Its static parts are below.
        // UNCOMPILED: not translated, it uses CompiledSection, which isn't translated (it uses Minecraft's `BlockEntity`).
        // CompiledSection.hasTransparencyState(0): not translated, its class isn't translated (it uses Minecraft's `BlockEntity`).
        // CompiledSection.getBlockEntities(0): not translated, its class isn't translated (it uses Minecraft's `BlockEntity`).
    }

    /// `CompileResult` (net/vulkanmod/render/chunk/build/task/CompileResult.java)
    pub mod compile_result {
        use super::*;
        // The class itself isn't translated: it uses RenderSection, which isn't translated (it holds an object of its own class (a linked structure)). Its static parts are below.
        // CompileResult.CompileResult(2): not translated, its class isn't translated (it uses RenderSection, which isn't translated (it holds an object of its own class (a linked structure))).
        // CompileResult.updateSection(0): not translated, its class isn't translated (it uses RenderSection, which isn't translated (it holds an object of its own class (a linked structure))).
    }

    /// `SortTransparencyTask` (net/vulkanmod/render/chunk/build/task/SortTransparencyTask.java)
    pub mod sort_transparency_task {
        use super::*;
        // The class itself isn't translated: it extends `ChunkTask`. Its static parts are below.
        // SortTransparencyTask.SortTransparencyTask(2): not translated, its class isn't translated (it extends `ChunkTask`).
        // SortTransparencyTask.name(0): not translated, its class isn't translated (it extends `ChunkTask`).
        // SortTransparencyTask.runTask(1): not translated, its class isn't translated (it extends `ChunkTask`).
    }

    /// `TaskDispatcher` (net/vulkanmod/render/chunk/build/task/TaskDispatcher.java)
    pub mod task_dispatcher {
        use super::*;
        // The class itself isn't translated: it uses `Thread`. Its static parts are below.
        // TaskDispatcher.TaskDispatcher(0): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.createThreads(0): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.createThreads(1): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.runTaskThread(1): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.schedule(1): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.pollTask(0): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.stopThreads(0): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.updateSections(0): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.scheduleSectionUpdate(1): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.doSectionUpdate(1): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.isIdle(0): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.clearBatchQueue(0): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.getStats(0): not translated, its class isn't translated (it uses `Thread`).
        // TaskDispatcher.getResourcesArray(0): not translated, its class isn't translated (it uses `Thread`).
    }

    /// `BuilderResources` (net/vulkanmod/render/chunk/build/thread/BuilderResources.java)
    pub mod builder_resources {
        use super::*;
        // The class itself isn't translated: it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)). Its static parts are below.
        // BuilderResources.BuilderResources(0): not translated, its class isn't translated (it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects))).
        // BuilderResources.update(2): not translated, its class isn't translated (it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects))).
        // BuilderResources.free(0): not translated, its class isn't translated (it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects))).
        // BuilderResources.updateBuildStats(1): not translated, its class isn't translated (it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects))).
        // BuilderResources.getRegion(0): not translated, its class isn't translated (it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects))).
        // BuilderResources.getTotalBuildTime(0): not translated, its class isn't translated (it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects))).
        // BuilderResources.getBuildCount(0): not translated, its class isn't translated (it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects))).
        // BuilderResources.resetCounters(0): not translated, its class isn't translated (it uses TintCache, which isn't translated (it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects))).
    }

    /// `ThreadBuilderPack` (net/vulkanmod/render/chunk/build/thread/ThreadBuilderPack.java)
    pub mod thread_builder_pack {
        use super::*;
        // The class itself isn't translated: it uses TerrainRenderType, which isn't translated (its constants carry values). Its static parts are below.
        // terrainBuilderConstructor: not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // ThreadBuilderPack.defaultTerrainBuilderConstructor(0): not translated, it uses ThreadBuilderPack.terrainBuilderConstructor, which isn't translated (it uses TerrainRenderType, which isn't translated (its constants carry values)).
        // ThreadBuilderPack.setTerrainBuilderConstructor(1): not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // ThreadBuilderPack.ThreadBuilderPack(0): not translated, its class isn't translated (it uses TerrainRenderType, which isn't translated (its constants carry values)).
        // ThreadBuilderPack.builder(1): not translated, its class isn't translated (it uses TerrainRenderType, which isn't translated (its constants carry values)).
        // ThreadBuilderPack.freeAll(0): not translated, its class isn't translated (it uses TerrainRenderType, which isn't translated (its constants carry values)).
    }

    /// `UploadBuffer` (net/vulkanmod/render/chunk/build/UploadBuffer.java)
    pub mod upload_buffer {
        use super::*;
        // The class itself isn't translated: it uses `ByteBuffer` (java.nio). Its static parts are below.
        // UploadBuffer.UploadBuffer(2): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // UploadBuffer.indexCount(0): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // UploadBuffer.getVertexBuffers(0): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // UploadBuffer.getIndexBuffer(0): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // UploadBuffer.release(0): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
    }

    /// `ChunkArea` (net/vulkanmod/render/chunk/ChunkArea.java)
    pub mod chunk_area {
        use super::*;
        // The class itself isn't translated: it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)). Its static parts are below.
        // ChunkArea.ChunkArea(3): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.updateFrustum(1): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.getFrustumIndex(1): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.getFrustumIndex(3): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.inFrustum(1): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.getFrustumBuffer(0): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.getDrawBuffers(0): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.resetQueue(0): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.setPosition(3): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.getPosition(0): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.addSection(0): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.removeSection(0): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.releaseBuffers(0): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // ChunkArea.free(0): not translated, its class isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
    }

    /// `ChunkAreaManager` (net/vulkanmod/render/chunk/ChunkAreaManager.java)
    pub mod chunk_area_manager {
        use super::*;
        // The class itself isn't translated: it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))). Its static parts are below.
        pub const WIDTH: i32 = 8;
        pub const HEIGHT: i32 = 8;
        pub const AREA_SIZE: i32 = 512;
        // AREA_SH_XZ: not translated, it calls Util.flooredLog, which isn't translated.
        // AREA_SH_Y: not translated, it calls Util.flooredLog, which isn't translated.
        pub const SEC_SH: i32 = 4;
        // BLOCK_TO_AREA_SH_XZ: not translated, it uses ChunkAreaManager.AREA_SH_XZ, which isn't translated (it calls Util.flooredLog, which isn't translated).
        // BLOCK_TO_AREA_SH_Y: not translated, it uses ChunkAreaManager.AREA_SH_Y, which isn't translated (it calls Util.flooredLog, which isn't translated).
        // ChunkAreaManager.ChunkAreaManager(3): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.repositionAreas(2): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.moveArea(5): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.getChunkArea(4): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.getChunkArea(1): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.updateFrustumVisibility(1): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.resetQueues(0): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.getAreaIndex(3): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.freeAllBuffers(0): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
        // ChunkAreaManager.getStats(0): not translated, its class isn't translated (it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml)))).
    }

    /// `ChunkStatusMap` (net/vulkanmod/render/chunk/ChunkStatusMap.java)
    pub mod chunk_status_map {
        use super::*;
        // The class itself isn't translated: it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs). Its static parts are below.
        pub const DATA_READY: i8 = 1i8;
        pub const LIGHT_READY: i8 = 2i8;
        pub const NEIGHBOURS_READY: i8 = 4i8;
        pub const CHUNK_READY: i8 = 3i8;
        pub const ALL_FLAGS: i8 = 7i8;
        // INSTANCE: not translated, it uses ChunkStatusMap, which isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.createInstance(1): not translated, it uses ChunkStatusMap, which isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.ChunkStatusMap(1): not translated, its class isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.updateDistance(1): not translated, its class isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.setChunkStatus(3): not translated, its class isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.resetChunkStatus(3): not translated, its class isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.updateNeighbours(2): not translated, its class isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.checkNeighbours(2): not translated, its class isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.chunkRenderReady(2): not translated, its class isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // ChunkStatusMap.reset(0): not translated, its class isn't translated (it uses `Long2ByteOpenHashMap` (it.unimi.dsi.fastutil.longs)).
    }

    /// `QuadFacing` (net/vulkanmod/render/chunk/cull/QuadFacing.java)
    pub mod quad_facing {
        use super::*;
        // VALUES: not translated, it calls QuadFacing.values, which isn't in the mod's source.
        // COUNT: not translated, it uses QuadFacing.VALUES, which isn't translated (it calls QuadFacing.values, which isn't in the mod's source).
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum QuadFacing {
            XPos,
            YPos,
            ZPos,
            XNeg,
            ZNeg,
            Undefined,
            YNeg,
        }
        impl QuadFacing {
            pub const VALUES: [QuadFacing; 7] = [QuadFacing::XPos, QuadFacing::YPos, QuadFacing::ZPos, QuadFacing::XNeg, QuadFacing::ZNeg, QuadFacing::Undefined, QuadFacing::YNeg, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    QuadFacing::XPos => "X_POS",
                    QuadFacing::YPos => "Y_POS",
                    QuadFacing::ZPos => "Z_POS",
                    QuadFacing::XNeg => "X_NEG",
                    QuadFacing::ZNeg => "Z_NEG",
                    QuadFacing::Undefined => "UNDEFINED",
                    QuadFacing::YNeg => "Y_NEG",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "X_POS" => QuadFacing::XPos,
                    "Y_POS" => QuadFacing::YPos,
                    "Z_POS" => QuadFacing::ZPos,
                    "X_NEG" => QuadFacing::XNeg,
                    "Z_NEG" => QuadFacing::ZNeg,
                    "UNDEFINED" => QuadFacing::Undefined,
                    "Y_NEG" => QuadFacing::YNeg,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
        pub fn from_direction(direction: cycling_option_widget_button_direction::Direction) -> quad_facing::QuadFacing {
            return match direction {
                DOWN => quad_facing::QuadFacing::YNeg,
                UP => quad_facing::QuadFacing::YPos,
                NORTH => quad_facing::QuadFacing::ZNeg,
                SOUTH => quad_facing::QuadFacing::ZPos,
                WEST => quad_facing::QuadFacing::XNeg,
                EAST => quad_facing::QuadFacing::XPos,
                _ => {
                panic!("MatchException");
                },
            };
        }
        // QuadFacing.fromNormal(1): not translated, it's overloaded with the same number of parameters.
        // QuadFacing.fromNormal(1): not translated, it's overloaded with the same number of parameters.
        // QuadFacing.fromNormal(3): not translated, it calls Minecraft's `Mth`.equal.
    }

    /// `FrustumOctree` (net/vulkanmod/render/chunk/frustum/FrustumOctree.java)
    pub mod frustum_octree {
        use super::*;
        pub const LEVELS: i32 = 2;
        #[derive(Clone, Debug)]
        pub struct FrustumOctree {
        }
        impl FrustumOctree {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // FrustumOctree.updateFrustumVisibility(2): not translated, it uses ChunkArea, which isn't translated (it uses DrawBuffers, which isn't translated (it uses `Vector3i` (org.joml))).
        // FrustumOctree.innerCube(8): not translated, it's overloaded with the same number of parameters.
        // FrustumOctree.lastInnerCube(7): not translated, it's overloaded with the same number of parameters.
        pub fn get_offset(base_shift: i32, x: i32, y: i32, z: i32) -> i32 {
            return ((x << (2 + base_shift)) + (y << (1 + base_shift))) + (z << base_shift);
        }
        pub fn fill_result_buffer(buffer: &mut [i8], begin_idx: i32, end_idx: i32, result: i8) {
            for i in begin_idx..end_idx {
                buffer[i as usize] = result;
            }
        }
    }

    /// `VFrustum` (net/vulkanmod/render/chunk/frustum/VFrustum.java)
    pub mod v_frustum {
        use super::*;
        // The class itself isn't translated: it uses `Vector4f` (org.joml). Its static parts are below.
        // VFrustum.offsetToFullyIncludeCameraCube(1): not translated, its class isn't translated (it uses `Vector4f` (org.joml)).
        // VFrustum.setCamOffset(3): not translated, its class isn't translated (it uses `Vector4f` (org.joml)).
        // VFrustum.calculateFrustum(2): not translated, its class isn't translated (it uses `Vector4f` (org.joml)).
        // VFrustum.cubeInFrustum(6): not translated, its class isn't translated (it uses `Vector4f` (org.joml)).
        // VFrustum.testFrustum(6): not translated, its class isn't translated (it uses `Vector4f` (org.joml)).
        // VFrustum.intersectAab(6): not translated, its class isn't translated (it uses `Vector4f` (org.joml)).
        // VFrustum.isVisible(1): not translated, its class isn't translated (it uses `Vector4f` (org.joml)).
        // VFrustum.cubeInFrustum(6): not translated, its class isn't translated (it uses `Vector4f` (org.joml)).
    }

    /// `GraphDirections` (net/vulkanmod/render/chunk/graph/GraphDirections.java)
    pub mod graph_directions {
        use super::*;
        pub const DOWN: i32 = 0;
        pub const UP: i32 = 1;
        pub const NORTH: i32 = 2;
        pub const SOUTH: i32 = 3;
        pub const WEST: i32 = 4;
        pub const EAST: i32 = 5;
        #[derive(Clone, Debug)]
        pub struct GraphDirections {
        }
        impl GraphDirections {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
    }

    /// `SectionGraph` (net/vulkanmod/render/chunk/graph/SectionGraph.java)
    pub mod section_graph {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `Minecraft`. Its static parts are below.
        // SectionGraph.SectionGraph(3): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.update(3): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.initializeQueueForFullUpdate(1): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.initFirstNode(2): not translated, it uses RenderSection, which isn't translated (it holds an object of its own class (a linked structure)).
        pub fn init_visibility() -> i64 {
            let mut vis: i64 = 0i64;
            for dir in 0..6 {
                vis |= 1i64 << (48 + dir);
                vis |= 1i64 << (56 + dir);
            }
            return vis;
        }
        // SectionGraph.initUpdate(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.resetUpdateQueues(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.updateRenderChunks(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.scheduleRebuilds(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.notInFrustum(1): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.visitAdjacentNodes(2): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.checkToAdd(5): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.updateRenderChunksSpectator(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.addNode(4): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.getChunkAreaQueue(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.getSectionQueue(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.getBlockEntitiesSections(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.getLastFrame(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // SectionGraph.getStatistics(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
    }

    /// `RenderSection` (net/vulkanmod/render/chunk/RenderSection.java)
    pub mod render_section {
        use super::*;
        // The class itself isn't translated: it holds an object of its own class (a linked structure). Its static parts are below.
        // RenderSection.RenderSection(4): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setOrigin(3): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.addDir(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setDirections(2): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setDirectionChanges(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.hasDirection(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.hasMainDirection(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setAdjacent(2): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.resetAdjacent(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.addAdjDir(2): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.removeAdjDir(2): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.resortTransparency(2): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.rebuildChunkAsync(3): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.rebuildChunkSync(2): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.createCompileTask(2): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.cancelTasks(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setNotDirty(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.isDirty(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.isDirtyFromPlayer(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.xOffset(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.yOffset(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.zOffset(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.resetDrawParameters(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setChunkArea(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.getChunkArea(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.getCompiledSection(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.isCompiled(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setVisibility(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setCompletelyEmpty(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setContainsBlockEntities(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.getDirections(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.getVisibilityDirs(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.getVisibility(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.isCompletelyEmpty(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.containsBlockEntities(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.updateGlobalBlockEntities(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.reset(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.resetDrawParameters(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setDirty(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setCompiledSection(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setLastFrame(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.setLastFrame2(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.getLastFrame(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // RenderSection.getLastFrame2(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
    }

    /// `RenderSection.CompileStatus` (net/vulkanmod/render/chunk/RenderSection.java)
    pub mod render_section_compile_status {
        use super::*;
        // The class itself isn't translated: it uses CompiledSection, which isn't translated (it uses Minecraft's `BlockEntity`). Its static parts are below.
    }

    /// `SectionGrid` (net/vulkanmod/render/chunk/SectionGrid.java)
    pub mod section_grid {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `Level`. Its static parts are below.
        // SectionGrid.SectionGrid(2): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.createChunks(0): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.freeAllBuffers(0): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.getChunkIndex(3): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.setViewDistance(1): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.repositionCamera(2): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.moveSection(9): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.setNeighbours(8): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.unsetNeighbours(1): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.setYNeighbours(0): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.setYNeighbours(4): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.setChunkArea(4): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.setDirty(4): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.getSectionAtBlockPos(1): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.getSectionAtBlockPos(3): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.getSectionAtSectionPos(3): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.getRenderSectionsAt(2): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.updateFrustumVisibility(1): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.getChunkAreaManager(0): not translated, its class isn't translated (it uses Minecraft's `Level`).
        // SectionGrid.getSectionCount(0): not translated, its class isn't translated (it uses Minecraft's `Level`).
    }

    /// `AreaSetQueue` (net/vulkanmod/render/chunk/util/AreaSetQueue.java)
    pub mod area_set_queue {
        use super::*;
        // The class itself isn't translated: it uses StaticQueue, which isn't translated (it's generic). Its static parts are below.
        // AreaSetQueue.AreaSetQueue(3): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
        // AreaSetQueue.AreaSetQueue(1): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
        // AreaSetQueue.add(1): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
        // AreaSetQueue.clear(0): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
        // AreaSetQueue.iterator(1): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
        // AreaSetQueue.iterator(0): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
        // AreaSetQueue.size(0): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
        // AreaSetQueue.set(0): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
        // AreaSetQueue.queue(0): not translated, its class isn't translated (it uses StaticQueue, which isn't translated (it's generic)).
    }

    /// `BufferUtil` (net/vulkanmod/render/chunk/util/BufferUtil.java)
    pub mod buffer_util {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct BufferUtil {
        }
        impl BufferUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // BufferUtil.clone(1): not translated, it uses `ByteBuffer` (java.nio).
        // BufferUtil.bufferSlice(3): not translated, it uses `ByteBuffer` (java.nio).
    }

    /// `CircularIntList` (net/vulkanmod/render/chunk/util/CircularIntList.java)
    pub mod circular_int_list {
        use super::*;
        // The class itself isn't translated: it uses OwnIterator, which isn't translated (it implements `Iterator` (java.util)). Its static parts are below.
        // CircularIntList.CircularIntList(1): not translated, its class isn't translated (it uses OwnIterator, which isn't translated (it implements `Iterator` (java.util))).
        // CircularIntList.updateStartIdx(1): not translated, its class isn't translated (it uses OwnIterator, which isn't translated (it implements `Iterator` (java.util))).
        // CircularIntList.getNext(1): not translated, its class isn't translated (it uses OwnIterator, which isn't translated (it implements `Iterator` (java.util))).
        // CircularIntList.getPrevious(1): not translated, its class isn't translated (it uses OwnIterator, which isn't translated (it implements `Iterator` (java.util))).
        // CircularIntList.iterator(0): not translated, its class isn't translated (it uses OwnIterator, which isn't translated (it implements `Iterator` (java.util))).
        // CircularIntList.getRangeIterator(2): not translated, its class isn't translated (it uses OwnIterator, which isn't translated (it implements `Iterator` (java.util))).
        // CircularIntList.createRangeIterator(0): not translated, its class isn't translated (it uses OwnIterator, which isn't translated (it implements `Iterator` (java.util))).
    }

    /// `CircularIntList.OwnIterator` (net/vulkanmod/render/chunk/util/CircularIntList.java)
    pub mod circular_int_list_own_iterator {
        use super::*;
        // The class itself isn't translated: it implements `Iterator` (java.util). Its static parts are below.
        // CircularIntList.OwnIterator.hasNext(0): not translated, its class isn't translated (it implements `Iterator` (java.util)).
        // CircularIntList.OwnIterator.next(0): not translated, its class isn't translated (it implements `Iterator` (java.util)).
        // CircularIntList.OwnIterator.getCurrentIndex(0): not translated, its class isn't translated (it implements `Iterator` (java.util)).
        // CircularIntList.OwnIterator.restart(0): not translated, its class isn't translated (it implements `Iterator` (java.util)).
    }

    /// `CircularIntList.RangeIterator` (net/vulkanmod/render/chunk/util/CircularIntList.java)
    pub mod circular_int_list_range_iterator {
        use super::*;
        // The class itself isn't translated: it implements `Iterator` (java.util). Its static parts are below.
        // CircularIntList.RangeIterator.update(2): not translated, its class isn't translated (it implements `Iterator` (java.util)).
        // CircularIntList.RangeIterator.hasNext(0): not translated, its class isn't translated (it implements `Iterator` (java.util)).
        // CircularIntList.RangeIterator.next(0): not translated, its class isn't translated (it implements `Iterator` (java.util)).
        // CircularIntList.RangeIterator.getCurrentIndex(0): not translated, its class isn't translated (it implements `Iterator` (java.util)).
        // CircularIntList.RangeIterator.restart(0): not translated, its class isn't translated (it implements `Iterator` (java.util)).
    }

    /// `ResettableQueue` (net/vulkanmod/render/chunk/util/ResettableQueue.java)
    pub mod resettable_queue {
        use super::*;
        // The class itself isn't translated: it's generic. Its static parts are below.
        // ResettableQueue.ResettableQueue(0): not translated, its class isn't translated (it's generic).
        // ResettableQueue.ResettableQueue(1): not translated, its class isn't translated (it's generic).
        // ResettableQueue.hasNext(0): not translated, its class isn't translated (it's generic).
        // ResettableQueue.poll(0): not translated, its class isn't translated (it's generic).
        // ResettableQueue.add(1): not translated, its class isn't translated (it's generic).
        // ResettableQueue.ensureCapacity(1): not translated, its class isn't translated (it's generic).
        // ResettableQueue.resize(1): not translated, its class isn't translated (it's generic).
        // ResettableQueue.size(0): not translated, its class isn't translated (it's generic).
        // ResettableQueue.rewind(0): not translated, its class isn't translated (it's generic).
        // ResettableQueue.get(1): not translated, its class isn't translated (it's generic).
        // ResettableQueue.clear(0): not translated, its class isn't translated (it's generic).
        // ResettableQueue.iterator(1): not translated, its body couldn't be read: an anonymous class.
        // ResettableQueue.iterator(0): not translated, its class isn't translated (it's generic).
        // ResettableQueue.forEach(1): not translated, its class isn't translated (it's generic).
    }

    /// `SimpleDirection` (net/vulkanmod/render/chunk/util/SimpleDirection.java)
    pub mod simple_direction {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // VALUES: not translated, it uses SimpleDirection, which isn't translated (its constants carry values).
        // SimpleDirection.of(1): not translated, it uses SimpleDirection, which isn't translated (its constants carry values).
        // SimpleDirection.SimpleDirection(4): not translated, its class isn't translated (its constants carry values).
        // SimpleDirection.get3DDataValue(0): not translated, its class isn't translated (its constants carry values).
        // SimpleDirection.getStepX(0): not translated, its class isn't translated (its constants carry values).
        // SimpleDirection.getStepY(0): not translated, its class isn't translated (its constants carry values).
        // SimpleDirection.getStepZ(0): not translated, its class isn't translated (its constants carry values).
    }

    /// `StaticQueue` (net/vulkanmod/render/chunk/util/StaticQueue.java)
    pub mod static_queue {
        use super::*;
        // The class itself isn't translated: it's generic. Its static parts are below.
        // StaticQueue.StaticQueue(0): not translated, its class isn't translated (it's generic).
        // StaticQueue.StaticQueue(1): not translated, its class isn't translated (it's generic).
        // StaticQueue.hasNext(0): not translated, its class isn't translated (it's generic).
        // StaticQueue.poll(0): not translated, its class isn't translated (it's generic).
        // StaticQueue.add(1): not translated, its class isn't translated (it's generic).
        // StaticQueue.size(0): not translated, its class isn't translated (it's generic).
        // StaticQueue.clear(0): not translated, its class isn't translated (it's generic).
        // StaticQueue.iterator(1): not translated, its body couldn't be read: an anonymous class.
        // StaticQueue.iterator(0): not translated, its class isn't translated (it's generic).
        // StaticQueue.forEach(1): not translated, its class isn't translated (it's generic).
    }

    /// `Util` (net/vulkanmod/render/chunk/util/Util.java)
    pub mod util {
        use super::*;
        pub static DIRECTIONS: LazyLock<Vec<cycling_option_widget_button_direction::Direction>> = LazyLock::new(|| cycling_option_widget_button_direction::Direction::VALUES.to_vec());
        // XZ_DIRECTIONS: not translated, it calls Util.getXzDirections, which isn't translated.
        #[derive(Clone, Debug)]
        pub struct Util {
        }
        impl Util {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn get_opposite_dir_idx(idx: i8) -> i8 {
            return ((if ((idx as i32) & 1) != 0 { (idx as i32) - 1 } else { (idx as i32) + 1 }) as i8);
        }
        // Util.getXzDirections(0): not translated, a CyclingOptionWidget.Button.Direction with no value (null in Java).
        pub fn pos_long_hash(x: i32, y: i32, z: i32) -> i64 {
            return (((x as i64) & 65535i64) | (((z as i64) << 16) & 4294901760i64)) | (((y as i64) << 32) & 281470681743360i64);
        }
        // Util.flooredLog(1): not translated, its body couldn't be read: an assert.
        pub fn align(l: i64, alignment: i32) -> i64 {
            if alignment == 0 {
                return l;
            }
            let r: i64 = l % (alignment as i64);
            return if r != 0i64 { (l + (alignment as i64)) - r } else { l };
        }
    }

    /// `WorldRenderer` (net/vulkanmod/render/chunk/WorldRenderer.java)
    pub mod world_renderer {
        use super::*;
        // The class itself isn't translated: it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs)). Its static parts are below.
        // INSTANCE: not translated, it uses WorldRenderer, which isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.init(5): not translated, it uses WorldRenderer, which isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.WorldRenderer(5): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.allocateIndirectBuffers(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.benchCallback(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.setupRenderer(4): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.uploadSections(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.isSectionCompiled(1): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.allChanged(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.setLevel(1): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.addOnAllChangedCallback(1): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.clearOnAllChangedCallbacks(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.renderSectionLayer(6): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.sortTranslucentSections(3): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.renderBlockEntities(4): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.resetSampler(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.setPartialTick(1): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.scheduleGraphUpdate(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.graphNeedsUpdate(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getVisibleSectionsCount(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.setSectionDirty(4): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getSectionGrid(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getChunkAreaManager(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getTaskDispatcher(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getLastFrame(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getRenderDistance(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getChunkStatistics(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.cleanUp(0): not translated, its class isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getInstance(0): not translated, it uses WorldRenderer, which isn't translated (it uses RenderRegionBuilder, which isn't translated (it uses `Long2ReferenceLinkedOpenHashMap` (it.unimi.dsi.fastutil.longs))).
        // WorldRenderer.getLevel(0): not translated, it uses Minecraft's `ClientLevel`.
        // WorldRenderer.getCameraPos(0): not translated, it uses `Vector3d` (org.joml).
    }

    /// `EGlProgram` (net/vulkanmod/render/engine/EGlProgram.java)
    pub mod e_gl_program {
        use super::*;
        // The class itself isn't translated: it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be)). Its static parts are below.
        // LOGGER: the logger. Log calls become comments.
        // BUILT_IN_UNIFORMS: not translated, it calls `Sets` (com.google.common.collect).newHashSet.
        // INVALID_PROGRAM: not translated, it uses EGlProgram, which isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
        // EGlProgram.EGlProgram(2): not translated, its class isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
        // EGlProgram.setupUniforms(3): not translated, it uses Minecraft's `UniformDescription`.
        // EGlProgram.getUniform(1): not translated, it calls Minecraft's `RenderSystem`.assertOnRenderThread.
        // EGlProgram.getProgramId(0): not translated, its class isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
        // EGlProgram.toString(0): not translated, its class isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
        // EGlProgram.getDebugLabel(0): not translated, its class isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
        // EGlProgram.getUniforms(0): not translated, its class isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
    }

    /// `VkBackend` (net/vulkanmod/render/engine/VkBackend.java)
    pub mod vk_backend {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `GpuBackend`. Its static parts are below.
        // VkBackend.getName(0): not translated, its class isn't translated (it implements Minecraft's `GpuBackend`).
        // VkBackend.setWindowHints(0): not translated, its class isn't translated (it implements Minecraft's `GpuBackend`).
        // VkBackend.handleWindowCreationErrors(1): not translated, its class isn't translated (it implements Minecraft's `GpuBackend`).
        // VkBackend.createDevice(3): not translated, its class isn't translated (it implements Minecraft's `GpuBackend`).
    }

    /// `VkCommandEncoder` (net/vulkanmod/render/engine/VkCommandEncoder.java)
    pub mod vk_command_encoder {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `CommandEncoderBackend`. Its static parts are below.
        // LOGGER: the logger. Log calls become comments.
        // VkCommandEncoder.VkCommandEncoder(1): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.createRenderPass(3): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.createRenderPass(5): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.isInRenderPass(0): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.clearColorTexture(2): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.clearColorAndDepthTextures(4): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.clearColorAndDepthTextures(8): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.clearDepthTexture(2): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.writeToBuffer(2): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.mapBuffer(3): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.copyToBuffer(2): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.writeToTexture(10): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.writeToTexture(9): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.copyTextureToBuffer(5): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.copyTextureToBuffer(9): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.copyTextureToTexture(9): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.createFence(0): not translated, its body couldn't be read: an anonymous class.
        // VkCommandEncoder.timerQueryBegin(0): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.timerQueryEnd(1): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.presentTexture(1): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.executeDrawMultiple(6): not translated, it's generic.
        // VkCommandEncoder.executeDraw(6): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.drawFromBuffers(7): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.trySetup(1): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.setupUniforms(1): not translated, its body couldn't be read: an assert.
        // VkCommandEncoder.bindPipeline(1): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.applyPipelineState(1): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.finishRenderPass(1): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
        // VkCommandEncoder.getDevice(0): not translated, its class isn't translated (it implements Minecraft's `CommandEncoderBackend`).
    }

    /// `VkConst` (net/vulkanmod/render/engine/VkConst.java)
    pub mod vk_const {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct VkConst {
        }
        impl VkConst {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // VkConst.of(1): not translated, it's overloaded with the same number of parameters.
        // VkConst.of(1): not translated, it's overloaded with the same number of parameters.
    }

    /// `VkDebugLabel` (net/vulkanmod/render/engine/VkDebugLabel.java)
    pub mod vk_debug_label {
        use super::*;
        // LOGGER: the logger. Log calls become comments.
        #[derive(Clone, Debug)]
        pub struct VkDebugLabel {
        }
        impl VkDebugLabel {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
            // VkDebugLabel.applyLabel(1): not translated, it's overloaded with the same number of parameters.
            // VkDebugLabel.applyLabel(1): not translated, it's overloaded with the same number of parameters.
            // VkDebugLabel.applyLabel(1): not translated, it's overloaded with the same number of parameters.
            // VkDebugLabel.applyLabel(1): not translated, it's overloaded with the same number of parameters.
            // VkDebugLabel.applyLabel(1): not translated, it's overloaded with the same number of parameters.
            pub fn exists(&self) -> bool {
                return true;
            }
        }
        pub fn create(bl: bool, set: &JSet<String>) -> vk_debug_label::VkDebugLabel {
            return vk_debug_label::VkDebugLabel::new();
        }
    }

    /// `VkFbo` (net/vulkanmod/render/engine/VkFbo.java)
    pub mod vk_fbo {
        use super::*;
        // The class itself isn't translated: it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`). Its static parts are below.
        // VkFbo.VkFbo(2): not translated, its class isn't translated (it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`)).
        // VkFbo.bind(0): not translated, its class isn't translated (it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`)).
        // VkFbo.clearAttachments(0): not translated, its class isn't translated (it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`)).
        // VkFbo.close(0): not translated, its class isn't translated (it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`)).
        // VkFbo.needsClear(0): not translated, its class isn't translated (it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`)).
    }

    /// `VkGpuBuffer` (net/vulkanmod/render/engine/VkGpuBuffer.java)
    pub mod vk_gpu_buffer {
        use super::*;
        // The class itself isn't translated: it extends Minecraft's `GpuBuffer`. Its static parts are below.
        // VkGpuBuffer.VkGpuBuffer(4): not translated, its class isn't translated (it extends Minecraft's `GpuBuffer`).
        // VkGpuBuffer.VkGpuBuffer(1): not translated, its class isn't translated (it extends Minecraft's `GpuBuffer`).
        // VkGpuBuffer.isClosed(0): not translated, its class isn't translated (it extends Minecraft's `GpuBuffer`).
        // VkGpuBuffer.close(0): not translated, its class isn't translated (it extends Minecraft's `GpuBuffer`).
        // VkGpuBuffer.getBuffer(0): not translated, its class isn't translated (it extends Minecraft's `GpuBuffer`).
        // VkGpuBuffer.setOffset(1): not translated, its class isn't translated (it extends Minecraft's `GpuBuffer`).
        pub fn buffer_usage_to_gl_enum(i: i32) -> i32 {
            let stream: bool = (i & 4) != 0;
            if (i & 2) != 0 {
                return if stream { 35040 } else { 35044 };
            } else if (i & 1) != 0 {
                return if stream { 35041 } else { 35045 };
            } else {
                return 35044;
            }
            #[allow(unreachable_code)]
            unreachable!()
        }
        // VkGpuBuffer.from(1): not translated, it uses VkGpuBuffer, which isn't translated (it extends Minecraft's `GpuBuffer`).
    }

    /// `VkGpuBuffer.MappedView` (net/vulkanmod/render/engine/VkGpuBuffer.java)
    pub mod vk_gpu_buffer_mapped_view {
        use super::*;
        // The class itself isn't translated: it implements `MappedView`. Its static parts are below.
        // VkGpuBuffer.MappedView.MappedView(2): not translated, its class isn't translated (it implements `MappedView`).
        // VkGpuBuffer.MappedView.data(0): not translated, its class isn't translated (it implements `MappedView`).
        // VkGpuBuffer.MappedView.close(0): not translated, its class isn't translated (it implements `MappedView`).
    }

    /// `VkGpuDevice` (net/vulkanmod/render/engine/VkGpuDevice.java)
    pub mod vk_gpu_device {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `GpuDeviceBackend`. Its static parts are below.
        // LOGGER: the logger. Log calls become comments.
        // VkGpuDevice.VkGpuDevice(3): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.debugLabels(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.createCommandEncoder(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.createSampler(6): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.createTexture(7): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.createTexture(7): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.gpuTextureFromVulkanImage(1): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.createTextureView(1): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.createTextureView(3): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.createBuffer(3): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.createBuffer(3): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getImplementationInformation(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getLastDebugMessages(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.isDebuggingEnabled(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getRenderer(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getVendor(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getBackendName(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getVersion(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getMaxSupportedTextureSize(0): not translated, it calls Minecraft's `GlStateManager`._getInteger.
        // VkGpuDevice.getMaxTextureSize(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getUniformOffsetAlignment(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.clearPipelineCache(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getEnabledExtensions(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getMaxSupportedAnisotropy(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.close(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.setVsync(1): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.presentFrame(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.isZZeroToOne(0): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getOrCompileShader(4): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.getCachedShaderSrc(4): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.precompilePipeline(2): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.compilePipeline(1): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.compileShader(2): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
        // VkGpuDevice.compilePipeline(2): not translated, its class isn't translated (it implements Minecraft's `GpuDeviceBackend`).
    }

    /// `VkGpuDevice.ShaderCompilationKey` (net/vulkanmod/render/engine/VkGpuDevice.java)
    pub mod vk_gpu_device_shader_compilation_key {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `ShaderType`. Its static parts are below.
        // VkGpuDevice.ShaderCompilationKey.ShaderCompilationKey(3): not translated, its class isn't translated (it uses Minecraft's `ShaderType`).
        // VkGpuDevice.ShaderCompilationKey.toString(0): not translated, its class isn't translated (it uses Minecraft's `ShaderType`).
        // VkGpuDevice.ShaderCompilationKey.id(0): not translated, its class isn't translated (it uses Minecraft's `ShaderType`).
        // VkGpuDevice.ShaderCompilationKey.type(0): not translated, its class isn't translated (it uses Minecraft's `ShaderType`).
        // VkGpuDevice.ShaderCompilationKey.defines(0): not translated, its class isn't translated (it uses Minecraft's `ShaderType`).
    }

    /// `VkGpuDevice.VkRenderPipeline` (net/vulkanmod/render/engine/VkGpuDevice.java)
    pub mod vk_gpu_device_vk_render_pipeline {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `CompiledRenderPipeline`. Its static parts are below.
        // VkGpuDevice.VkRenderPipeline.VkRenderPipeline(1): not translated, its class isn't translated (it implements Minecraft's `CompiledRenderPipeline`).
        // VkGpuDevice.VkRenderPipeline.isValid(0): not translated, its class isn't translated (it implements Minecraft's `CompiledRenderPipeline`).
    }

    /// `VkGpuTexture` (net/vulkanmod/render/engine/VkGpuTexture.java)
    pub mod vk_gpu_texture {
        use super::*;
        // The class itself isn't translated: it extends Minecraft's `GlTexture`. Its static parts are below.
        // glToVkMap: not translated, it uses `Reference2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects).
        // VkGpuTexture.VkGpuTexture(9): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.close(0): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.isClosed(0): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.glId(0): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.setClearColor(1): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.setDepthClearValue(1): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.needsClear(0): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.getFbo(1): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.getVulkanImage(0): not translated, its class isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.fromGlTexture(1): not translated, it uses VkGpuTexture, which isn't translated (it extends Minecraft's `GlTexture`).
        // VkGpuTexture.textureFormat(1): not translated, it uses Minecraft's `TextureFormat`.
        // VkGpuTexture.vkFormat(1): not translated, it uses Minecraft's `TextureFormat`.
        pub fn vk_image_view_type(usage: i32) -> i32 {
            let mut view_type: i32;
            if (usage & 16) != 0 {
                view_type = 3;
            } else {
                view_type = 1;
            }
            return view_type;
        }
    }

    /// `VkRenderPass` (net/vulkanmod/render/engine/VkRenderPass.java)
    pub mod vk_render_pass {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `RenderPassBackend`. Its static parts are below.
        pub const MAX_VERTEX_BUFFERS: i32 = 1;
        // VALIDATION: not translated, it uses Minecraft's `SharedConstants`.IS_RUNNING_IN_IDE.
        // VkRenderPass.VkRenderPass(3): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.hasDepthTexture(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.pushDebugGroup(1): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.popDebugGroup(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.setPipeline(1): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.bindTexture(3): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.setUniform(2): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.setUniform(2): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.enableScissor(4): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.disableScissor(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.isScissorEnabled(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.getScissorX(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.getScissorY(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.getScissorWidth(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.getScissorHeight(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.getScissorState(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.setVertexBuffer(2): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.setIndexBuffer(2): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.drawIndexed(4): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.drawMultipleIndexed(5): not translated, it's generic.
        // VkRenderPass.draw(2): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.close(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.isClosed(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
        // VkRenderPass.getPipeline(0): not translated, its class isn't translated (it implements Minecraft's `RenderPassBackend`).
    }

    /// `VkRenderPass.TextureViewAndSampler` (net/vulkanmod/render/engine/VkRenderPass.java)
    pub mod vk_render_pass_texture_view_and_sampler {
        use super::*;
        // The class itself isn't translated: it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`). Its static parts are below.
        // VkRenderPass.TextureViewAndSampler.TextureViewAndSampler(2): not translated, its class isn't translated (it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`)).
        // VkRenderPass.TextureViewAndSampler.view(0): not translated, its class isn't translated (it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`)).
        // VkRenderPass.TextureViewAndSampler.sampler(0): not translated, its class isn't translated (it uses VkTextureView, which isn't translated (it extends Minecraft's `GpuTextureView`)).
    }

    /// `VkSampler` (net/vulkanmod/render/engine/VkSampler.java)
    pub mod vk_sampler {
        use super::*;
        // The class itself isn't translated: it extends Minecraft's `GpuSampler`. Its static parts are below.
        // VkSampler.VkSampler(6): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.getId(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.getAddressModeU(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.getAddressModeV(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.getMinFilter(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.getMagFilter(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.getMaxAnisotropy(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.getMaxLod(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.close(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
        // VkSampler.isClosed(0): not translated, its class isn't translated (it extends Minecraft's `GpuSampler`).
    }

    /// `VkTextureView` (net/vulkanmod/render/engine/VkTextureView.java)
    pub mod vk_texture_view {
        use super::*;
        // The class itself isn't translated: it extends Minecraft's `GpuTextureView`. Its static parts are below.
        // VkTextureView.VkTextureView(3): not translated, its class isn't translated (it extends Minecraft's `GpuTextureView`).
        // VkTextureView.getFbo(1): not translated, its class isn't translated (it extends Minecraft's `GpuTextureView`).
        // VkTextureView.isClosed(0): not translated, its class isn't translated (it extends Minecraft's `GpuTextureView`).
        // VkTextureView.close(0): not translated, its class isn't translated (it extends Minecraft's `GpuTextureView`).
        // VkTextureView.texture(0): not translated, its class isn't translated (it extends Minecraft's `GpuTextureView`).
    }

    /// `CubeModel` (net/vulkanmod/render/model/CubeModel.java)
    pub mod cube_model {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct CubeModel {
            pub polygons: Option<Vec<cube_model_polygon::Polygon>>,
            pub min_x: f32,
            pub min_y: f32,
            pub min_z: f32,
            pub max_x: f32,
            pub max_y: f32,
            pub max_z: f32,
            pub vertices: Option<Vec<cube_model_vertex::Vertex>>,
        }
        impl CubeModel {
            pub fn new() -> Self {
                let mut s = Self { polygons: None, min_x: 0.0f32, min_y: 0.0f32, min_z: 0.0f32, max_x: 0.0f32, max_y: 0.0f32, max_z: 0.0f32, vertices: None };
                s
            }
            // CubeModel.setVertices(15): not translated, a CubeModel.Polygon with no value (null in Java).
            // CubeModel.transformVertices(1): not translated, JOML's Vector3f.mulPosition with 2 values.
            pub fn get_polygons(&self) -> Option<Vec<cube_model_polygon::Polygon>> {
                return self.polygons.clone();
            }
        }
    }

    /// `CubeModel.Polygon` (net/vulkanmod/render/model/CubeModel.java)
    pub mod cube_model_polygon {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Polygon {
            pub vertices: Option<Vec<cube_model_vertex::Vertex>>,
            pub normal: Option<JVec3>,
        }
        impl Polygon {
            pub fn new_2(vertices: &[cube_model_vertex::Vertex], normal: JVec3) -> Self {
                let mut s = Self { vertices: Some(vertices.to_vec()), normal: Some(normal) };
                s.vertices = Some(vertices.to_vec());
                s.normal = Some(normal);
                s
            }
            // CubeModel.Polygon.Polygon(9): not translated, it calls Direction.getUnitVec3f, which isn't in the mod's source.
            pub fn vertices(&self) -> Option<Vec<cube_model_vertex::Vertex>> {
                return self.vertices.clone();
            }
            pub fn normal(&self) -> Option<JVec3> {
                return self.normal.clone();
            }
        }
        // CubeModel.Polygon.mirrorFacing(1): not translated, it calls Direction.getAxis, which isn't in the mod's source.
    }

    /// `CubeModel.Vertex` (net/vulkanmod/render/model/CubeModel.java)
    pub mod cube_model_vertex {
        use super::*;
        pub const SCALE_FACTOR: f32 = 16.0f32;
        #[derive(Clone, Debug)]
        pub struct Vertex {
            pub pos: JVec3,
            pub transformed: JVec3,
            pub u: f32,
            pub v: f32,
        }
        impl Vertex {
            pub fn new_5(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
                let mut s = Self { pos: JVec3::new(0.0, 0.0, 0.0), transformed: JVec3::new(0.0, 0.0, 0.0), u: u, v: v };
                s.pos = JVec3::new(x / 16.0f32, y / 16.0f32, z / 16.0f32);
                s.transformed = JVec3::new(0.0, 0.0, 0.0);
                s.u = u;
                s.v = v;
                s
            }
            pub fn new_4(pos: JVec3, transformed: JVec3, u: f32, v: f32) -> Self {
                let mut s = Self { pos: pos, transformed: transformed, u: u, v: v };
                s.pos = pos;
                s.transformed = transformed;
                s.u = u;
                s.v = v;
                s
            }
            pub fn remap(&self, u: f32, v: f32) -> cube_model_vertex::Vertex {
                return cube_model_vertex::Vertex::new_4(self.pos, self.transformed, u, v);
            }
            pub fn pos(&self) -> JVec3 {
                return self.transformed;
            }
            pub fn u(&self) -> f32 {
                return self.u;
            }
            pub fn v(&self) -> f32 {
                return self.v;
            }
        }
    }

    /// `ModelQuad` (net/vulkanmod/render/model/quad/ModelQuad.java)
    pub mod model_quad {
        use super::*;
        // The class itself isn't translated: it implements `ModelQuadView`. Its static parts are below.
        pub const VERTEX_SIZE: i32 = 8;
        pub fn vertex_offset(vertex_index: i32) -> i32 {
            return vertex_index * 8;
        }
        // ModelQuad.getFlags(0): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getX(1): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getY(1): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getZ(1): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getColor(1): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getU(1): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getV(1): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getColorIndex(0): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getFacingDirection(0): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.lightFace(0): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getQuadFacing(0): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.getNormal(0): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.setX(2): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.setY(2): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.setZ(2): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.setU(2): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.setV(2): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.setFlags(1): not translated, its class isn't translated (it implements `ModelQuadView`).
        // ModelQuad.setSprite(1): not translated, its class isn't translated (it implements `ModelQuadView`).
    }

    /// `ModelQuadFlags` (net/vulkanmod/render/model/quad/ModelQuadFlags.java)
    pub mod model_quad_flags {
        use super::*;
        pub const IS_PARTIAL: i32 = 1;
        pub const IS_PARALLEL: i32 = 2;
        pub const IS_ALIGNED: i32 = 4;
        #[derive(Clone, Debug)]
        pub struct ModelQuadFlags {
        }
        impl ModelQuadFlags {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn contains(flags: i32, mask: i32) -> bool {
            return (flags & mask) != 0;
        }
        // ModelQuadFlags.getQuadFlags(2): not translated, its body couldn't be read: a labelled statement.
    }

    /// `ModelQuadView` (net/vulkanmod/render/model/quad/ModelQuadView.java)
    pub mod model_quad_view {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // ModelQuadView.getFlags(0): not translated, it has no body.
        // ModelQuadView.getX(1): not translated, it has no body.
        // ModelQuadView.getY(1): not translated, it has no body.
        // ModelQuadView.getZ(1): not translated, it has no body.
        // ModelQuadView.getColor(1): not translated, it has no body.
        // ModelQuadView.getU(1): not translated, it has no body.
        // ModelQuadView.getV(1): not translated, it has no body.
        // ModelQuadView.getColorIndex(0): not translated, it has no body.
        // ModelQuadView.getFacingDirection(0): not translated, it has no body.
        // ModelQuadView.lightFace(0): not translated, it has no body.
        // ModelQuadView.getQuadFacing(0): not translated, it has no body.
        // ModelQuadView.getNormal(0): not translated, it has no body.
        // ModelQuadView.isTinted(0): not translated, its class isn't translated (it's an interface).
    }

    /// `QuadUtils` (net/vulkanmod/render/model/quad/QuadUtils.java)
    pub mod quad_utils {
        use super::*;
        pub const DEFAULT_START_IDX: i8 = 0i8;
        pub const FLIPPED_START_IDX: i8 = 3i8;
        #[derive(Clone, Debug)]
        pub struct QuadUtils {
        }
        impl QuadUtils {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn get_iteration_start_idx_2(aos: &[f32], lms: &[i32]) -> i32 {
            let ao00_11: f32 = aos[0] + aos[2];
            let ao10_01: f32 = aos[1] + aos[3];
            if ao00_11 > ao10_01 {
                return 0;
            }
            if ao00_11 < ao10_01 {
                return 3;
            }
            let lm00_11: f32 = ((lms[0] + lms[2]) as f32);
            let lm10_01: f32 = ((lms[1] + lms[3]) as f32);
            return if lm00_11 >= lm10_01 { 3 } else { 0 };
        }
        pub fn get_iteration_start_idx_1(aos: &[f32]) -> i32 {
            let ao00_11: f32 = aos[0] + aos[2];
            let ao10_01: f32 = aos[1] + aos[3];
            return if ao00_11 >= ao10_01 { 0 } else { 3 };
        }
    }

    /// `BuildTimeProfiler` (net/vulkanmod/render/profiling/BuildTimeProfiler.java)
    pub mod build_time_profiler {
        use super::*;
        pub static BENCH: JCell<bool> = JCell::new(false);
        pub static START_TIME: JCell<i64> = JCell::new(0i64);
        pub static DELTA_TIME: JCell<f32> = JCell::new(0.0f32);
        #[derive(Clone, Debug)]
        pub struct BuildTimeProfiler {
        }
        impl BuildTimeProfiler {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn run_bench(building: bool) {
            if build_time_profiler::BENCH.get() {
                if build_time_profiler::START_TIME.get() == 0i64 {
                    build_time_profiler::START_TIME.set(j_nanos());
                }
                if !building {
                    build_time_profiler::DELTA_TIME.set(((j_nanos() - build_time_profiler::START_TIME.get()) as f32) * (1.0e-6f32));
                    build_time_profiler::BENCH.set(false);
                    build_time_profiler::START_TIME.set(0i64);
                }
            }
        }
        // BuildTimeProfiler.startBench(0): not translated, it calls Minecraft's `Minecraft`.getInstance.
        pub fn get_delta_time() -> f32 {
            return build_time_profiler::DELTA_TIME.get();
        }
    }

    /// `DebugEntryMemoryStats` (net/vulkanmod/render/profiling/DebugEntryMemoryStats.java)
    pub mod debug_entry_memory_stats {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `DebugScreenEntry`. Its static parts are below.
        pub static GROUP: LazyLock<JIdentifier> = LazyLock::new(|| JIdentifier::new("minecraft", "vk_memory"));
        // DebugEntryMemoryStats.display(4): not translated, its class isn't translated (it implements Minecraft's `DebugScreenEntry`).
    }

    /// `Profiler` (net/vulkanmod/render/profiling/Profiler.java)
    pub mod profiler {
        use super::*;
        // The class itself isn't translated: it uses `LongArrayList` (it.unimi.dsi.fastutil.longs). Its static parts are below.
        pub const DEBUG: bool = false;
        pub const FORCE_ACTIVE: bool = false;
        pub const NANOS_IN_MS: i32 = 1000000;
        pub const CONVERSION: f32 = 1000000.0f32;
        pub const INV_CONVERSION: f32 = 1.0e-6f32;
        pub const SAMPLE_COUNT: i32 = 200;
        pub static ACTIVE: JCell<bool> = JCell::new(false);
        // MAIN_PROFILER: not translated, it uses Profiler, which isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        // Profiler.getMainProfiler(0): not translated, it uses Profiler, which isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        pub fn set_active(b: bool) {
            profiler::ACTIVE.set(b);
        }
        // Profiler.Profiler(1): not translated, its class isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        // Profiler.push(1): not translated, its class isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        // Profiler.pushNodeStack(1): not translated, its class isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        // Profiler.pop(0): not translated, its class isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        // Profiler.start(0): not translated, its class isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        // Profiler.end(0): not translated, its class isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        // Profiler.round(0): not translated, its class isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
        // Profiler.getProfilerResults(0): not translated, its class isn't translated (it uses `LongArrayList` (it.unimi.dsi.fastutil.longs)).
    }

    /// `Profiler.Node` (net/vulkanmod/render/profiling/Profiler.java)
    pub mod profiler_node {
        use super::*;
        // The class itself isn't translated: it holds an object of its own class (a linked structure). Its static parts are below.
        // Profiler.Node.Node(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // Profiler.Node.setParent(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // Profiler.Node.addChild(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // Profiler.Node.push(1): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // Profiler.Node.updateResult(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // Profiler.Node.reset(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
        // Profiler.Node.toString(0): not translated, its class isn't translated (it holds an object of its own class (a linked structure)).
    }

    /// `Profiler.ProfilerResults` (net/vulkanmod/render/profiling/Profiler.java)
    pub mod profiler_profiler_results {
        use super::*;
        // The class itself isn't translated: it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects). Its static parts are below.
        // Profiler.ProfilerResults.update(2): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // Profiler.ProfilerResults.getResult(0): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // Profiler.ProfilerResults.getPartialResults(0): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
    }

    /// `Profiler.Result` (net/vulkanmod/render/profiling/Profiler.java)
    pub mod profiler_result {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Result_ {
            pub name: String,
            pub value: f32,
        }
        impl Result_ {
            pub fn new(name: &str) -> Self {
                let mut s = Self { name: String::new(), value: 0.0f32 };
                s.name = name.to_string();
                s
            }
            pub fn set_value(&mut self, value: f32) {
                self.value = value;
            }
        }
    }

    /// `ProfilerOverlay` (net/vulkanmod/render/profiling/ProfilerOverlay.java)
    pub mod profiler_overlay {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `Minecraft`. Its static parts are below.
        pub const POLL_PERIOD: i64 = 100000000i64;
        // INSTANCE: not translated, it uses ProfilerOverlay, which isn't translated (it uses Minecraft's `Minecraft`).
        pub static SHOULD_RENDER: JCell<bool> = JCell::new(false);
        // lastResults: not translated, it uses ProfilerResults, which isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        pub static LAST_POLL_TIME: JCell<i64> = JCell::new(0i64);
        pub static FRAMETIME: JCell<f32> = JCell::new(0.0f32);
        pub static BUILD_STATS: LazyLock<JCell<Option<String>>> = LazyLock::new(|| JCell::new(None));
        // ProfilerOverlay.ProfilerOverlay(1): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // ProfilerOverlay.createInstance(1): not translated, it uses Minecraft's `Minecraft`.
        pub fn toggle() {
            profiler_overlay::SHOULD_RENDER.set(!profiler_overlay::SHOULD_RENDER.get());
            profiler::set_active(profiler_overlay::SHOULD_RENDER.get());
        }
        pub fn on_key_press(key: i32) {
        }
        // ProfilerOverlay.render(1): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // ProfilerOverlay.buildInfo(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // ProfilerOverlay.updateResults(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
        // ProfilerOverlay.getBuildStats(0): not translated, its class isn't translated (it uses Minecraft's `Minecraft`).
    }

    /// `CustomRenderPipelines` (net/vulkanmod/render/shader/CustomRenderPipelines.java)
    pub mod custom_render_pipelines {
        use super::*;
        // pipelines: not translated, it uses Minecraft's `RenderPipeline`.
        // GUI_TRIANGLES_SNIPPET: not translated, it uses Minecraft's `Snippet`.
        // GUI_TRIANGLES: not translated, it uses Minecraft's `RenderPipeline`.
        #[derive(Clone, Debug)]
        pub struct CustomRenderPipelines {
        }
        impl CustomRenderPipelines {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // CustomRenderPipelines.register(1): not translated, it uses Minecraft's `RenderPipeline`.
    }

    /// `PipelineConfigs` (net/vulkanmod/render/shader/PipelineConfigs.java)
    pub mod pipeline_configs {
        use super::*;
        // TERRAIN_UB0: not translated, it calls Builder.addUniform, which isn't translated.
        // TERRAIN_UB1: not translated, it calls Builder.addUniform, which isn't translated.
        // TERRAIN_UB2: not translated, it calls Builder.setSize, which isn't translated.
        // TERRAIN_PC: not translated, it calls Builder.addUniform, which isn't translated.
        // TERRAIN: not translated, it uses PipelineConfig, which isn't translated (it uses `EnumMap` (java.util)).
        // TERRAIN_EARLY_Z_CONFIG: not translated, it uses PipelineConfig, which isn't translated (it uses `EnumMap` (java.util)).
        #[derive(Clone, Debug)]
        pub struct PipelineConfigs {
        }
        impl PipelineConfigs {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
    }

    /// `PipelineManager` (net/vulkanmod/render/shader/PipelineManager.java)
    pub mod pipeline_manager {
        use super::*;
        // terrainVertexFormat: not translated, it uses Minecraft's `VertexFormat`.
        // terrainShader: not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // terrainShaderEarlyZ: not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // fastBlitPipeline: not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // cloudsPipeline: not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // shaderGetter: not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        #[derive(Clone, Debug)]
        pub struct PipelineManager {
        }
        impl PipelineManager {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // PipelineManager.init(0): not translated, it uses Minecraft's `VertexFormat`.
        // PipelineManager.setDefaultTerrainShaderGetter(0): not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // PipelineManager.createBasicPipelines(0): not translated, it uses PipelineManager.terrainShader, which isn't translated (it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`)).
        // PipelineManager.createPipeline(4): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // PipelineManager.createPipeline(3): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // PipelineManager.getTerrainShader(1): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // PipelineManager.setShaderGetter(1): not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // PipelineManager.setTerrainVertexFormat(1): not translated, it uses Minecraft's `VertexFormat`.
        // PipelineManager.getTerrainVertexFormat(0): not translated, it uses Minecraft's `VertexFormat`.
        // PipelineManager.getTerrainDirectShader(1): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // PipelineManager.getTerrainIndirectShader(1): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // PipelineManager.getFastBlitPipeline(0): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // PipelineManager.getCloudsPipeline(0): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
        // PipelineManager.destroyPipelines(0): not translated, it uses PipelineManager.terrainShaderEarlyZ, which isn't translated (it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`)).
    }

    /// `ShaderLoadUtil` (net/vulkanmod/render/shader/ShaderLoadUtil.java)
    pub mod shader_load_util {
        use super::*;
        // RESOURCES_PATH: not translated, unknown static SPIRVUtils.class.
        // SHADERS_PATH: not translated, it uses ShaderLoadUtil.RESOURCES_PATH, which isn't translated (unknown static SPIRVUtils.class).
        // REMAPPED_SHADERS: not translated, it calls `Sets` (com.google.common.collect).newHashSet.
        #[derive(Clone, Debug)]
        pub struct ShaderLoadUtil {
        }
        impl ShaderLoadUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // ShaderLoadUtil.resolveShaderPath(1): not translated, it uses ShaderLoadUtil.SHADERS_PATH, which isn't translated (it uses ShaderLoadUtil.RESOURCES_PATH, which isn't translated (unknown static SPIRVUtils.class)).
        // ShaderLoadUtil.resolveShaderPath(2): not translated, String.formatted with 2 values.
        // ShaderLoadUtil.loadShader(2): not translated, it calls ShaderLoadUtil.loadShader, which isn't translated.
        // ShaderLoadUtil.loadShader(3): not translated, it's overloaded with the same number of parameters.
        // ShaderLoadUtil.getConfigFilePath(2): not translated, it uses ShaderLoadUtil.RESOURCES_PATH, which isn't translated (unknown static SPIRVUtils.class).
        // ShaderLoadUtil.getJsonConfig(2): not translated, String.formatted with 3 values.
        // ShaderLoadUtil.getShaderSource(2): not translated, it's overloaded with the same number of parameters.
        // ShaderLoadUtil.getShaderSource(2): not translated, it's overloaded with the same number of parameters.
        // ShaderLoadUtil.getShaderSource(2): not translated, it's overloaded with the same number of parameters.
        // ShaderLoadUtil.getShaderSource(4): not translated, it uses ShaderKind, which isn't translated (its constants carry values).
        // ShaderLoadUtil.getFileName(1): not translated, String.lastIndexOf with 1 values.
        pub fn remove_name_space(path: &str) -> String {
            let idx: i32 = j_index_of_char(&path, char::from_u32(58 as u32).unwrap_or('\0'));
            return if idx > (-1) { j_substring(&path, idx + 1, None) } else { path.to_string() };
        }
        // ShaderLoadUtil.splitPath(1): not translated, String.lastIndexOf with 1 values.
        // ShaderLoadUtil.getInputStream(1): not translated, it uses `InputStream` (java.io).
        pub fn inject_defines(shader_src: &str, defines: Option<Vec<String>>) -> String {
            let mut string_builder: String = String::new();
            for define in (defines.as_ref().expect("null")).iter() {
                string_builder.push_str("#define ");
                string_builder.push_str(define);
                string_builder.push_str(&(format!("{}", '\n')));
            }
            let i: i32 = j_index_of_char(&shader_src, char::from_u32(10 as u32).unwrap_or('\0'));
            let out: String = format!("{}\n{}\n{}", j_substring(&shader_src, 0, Some(i + 1)), string_builder, j_substring(&shader_src, i + 1, None));
            return out.to_string();
        }
    }

    /// `CloudRenderer` (net/vulkanmod/render/sky/CloudRenderer.java)
    pub mod cloud_renderer {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `CloudStatus`. Its static parts are below.
        pub static TEXTURE_LOCATION: LazyLock<JIdentifier> = LazyLock::new(|| JIdentifier::new("minecraft", "textures/environment/clouds.png"));
        pub const DIR_NEG_Y_BIT: i32 = 1;
        pub const DIR_POS_Y_BIT: i32 = 2;
        pub const DIR_NEG_X_BIT: i32 = 4;
        pub const DIR_POS_X_BIT: i32 = 8;
        pub const DIR_NEG_Z_BIT: i32 = 16;
        pub const DIR_POS_Z_BIT: i32 = 32;
        pub const Y_BELOW_CLOUDS: i8 = 0i8;
        pub const Y_ABOVE_CLOUDS: i8 = 1i8;
        pub const Y_INSIDE_CLOUDS: i8 = 2i8;
        pub const CELL_WIDTH: i32 = 12;
        pub const CELL_HEIGHT: i32 = 4;
        // CloudRenderer.CloudRenderer(0): not translated, its class isn't translated (it uses Minecraft's `CloudStatus`).
        // CloudRenderer.loadTexture(0): not translated, its class isn't translated (it uses Minecraft's `CloudStatus`).
        // CloudRenderer.renderClouds(7): not translated, its class isn't translated (it uses Minecraft's `CloudStatus`).
        // CloudRenderer.resetBuffer(0): not translated, its class isn't translated (it uses Minecraft's `CloudStatus`).
        // CloudRenderer.buildClouds(4): not translated, its class isn't translated (it uses Minecraft's `CloudStatus`).
        // CloudRenderer.putVertex(5): not translated, it uses Minecraft's `BufferBuilder`.
        // CloudRenderer.createCloudGrid(1): not translated, it uses Minecraft's `ResourceManager`.
    }

    /// `CloudRenderer.CloudGrid` (net/vulkanmod/render/sky/CloudRenderer.java)
    pub mod cloud_renderer_cloud_grid {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct CloudGrid {
            pub width: i32,
            pub pixels: Vec<i32>,
            pub render_faces: Vec<i8>,
        }
        impl CloudGrid {
            pub fn new(pixels: &[i32], width: i32) -> Self {
                let mut s = Self { width: width, pixels: Vec::new(), render_faces: Vec::new() };
                s.pixels = pixels.to_vec();
                s.width = width;
                s.render_faces = s.compute_render_faces();
                s
            }
            pub fn compute_render_faces(&self) -> Vec<i8> {
                let mut render_faces: Vec<i8> = vec![0i8; (self.pixels.len() as i32) as usize];
                for z in 0..self.width {
                    for x in 0..self.width {
                        let idx: i32 = self.get_idx(x, z);
                        let pixel: i32 = self.pixels[idx as usize];
                        if cloud_renderer_cloud_grid::has_color(pixel) {
                            let mut faces: i8 = 3i8;
                            let mut adj_pixel: i32 = self.get_texel_wrapped(x - 1, z);
                            if pixel != adj_pixel {
                                faces = (((faces as i32) | 4) as i8);
                            }
                            adj_pixel = self.get_texel_wrapped(x + 1, z);
                            if pixel != adj_pixel {
                                faces = (((faces as i32) | 8) as i8);
                            }
                            adj_pixel = self.get_texel_wrapped(x, z - 1);
                            if pixel != adj_pixel {
                                faces = (((faces as i32) | 16) as i8);
                            }
                            adj_pixel = self.get_texel_wrapped(x, z + 1);
                            if pixel != adj_pixel {
                                faces = (((faces as i32) | 32) as i8);
                            }
                            render_faces[idx as usize] = faces;
                        }
                    }
                }
                return render_faces.to_vec();
            }
            pub fn get_texel_wrapped(&self, mut x: i32, mut z: i32) -> i32 {
                if x < 0 {
                    x = self.width - 1;
                }
                if x > (self.width - 1) {
                    x = 0;
                }
                if z < 0 {
                    z = self.width - 1;
                }
                if z > (self.width - 1) {
                    z = 0;
                }
                return self.pixels[(self.get_idx(x, z)) as usize];
            }
            pub fn get_wrapped_idx(&self, mut x: i32, mut z: i32) -> i32 {
                x = x.rem_euclid(self.width);
                z = z.rem_euclid(self.width);
                return self.get_idx(x, z);
            }
            pub fn get_idx(&self, x: i32, z: i32) -> i32 {
                return (z * self.width) + x;
            }
        }
        pub fn has_color(pixel: i32) -> bool {
            return ((pixel >> 24) & 0xFF) > 1;
        }
    }

    /// `ImageUploadHelper` (net/vulkanmod/render/texture/ImageUploadHelper.java)
    pub mod image_upload_helper {
        use super::*;
        // The class itself isn't translated: it uses Queue, which isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)). Its static parts are below.
        // INSTANCE: not translated, it uses ImageUploadHelper, which isn't translated (it uses Queue, which isn't translated (it uses `VkQueue` (org.lwjgl.vulkan))).
        // ImageUploadHelper.submitCommands(0): not translated, its class isn't translated (it uses Queue, which isn't translated (it uses `VkQueue` (org.lwjgl.vulkan))).
        // ImageUploadHelper.submitCommands(1): not translated, its class isn't translated (it uses Queue, which isn't translated (it uses `VkQueue` (org.lwjgl.vulkan))).
        // ImageUploadHelper.getOrStartCommandBuffer(0): not translated, its class isn't translated (it uses Queue, which isn't translated (it uses `VkQueue` (org.lwjgl.vulkan))).
        // ImageUploadHelper.getCommandBuffer(0): not translated, its class isn't translated (it uses Queue, which isn't translated (it uses `VkQueue` (org.lwjgl.vulkan))).
    }

    /// `SpriteUpdateUtil` (net/vulkanmod/render/texture/SpriteUpdateUtil.java)
    pub mod sprite_update_util {
        use super::*;
        // transitionedLayouts: not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        #[derive(Clone, Debug)]
        pub struct SpriteUpdateUtil {
        }
        impl SpriteUpdateUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // SpriteUpdateUtil.addTransitionedLayout(1): not translated, it uses SpriteUpdateUtil.transitionedLayouts, which isn't translated (a Map or Set keyed by a VulkanImage).
        // SpriteUpdateUtil.transitionLayouts(0): not translated, it uses SpriteUpdateUtil.transitionedLayouts, which isn't translated (a Map or Set keyed by a VulkanImage).
    }

    /// `DrawUtil` (net/vulkanmod/render/util/DrawUtil.java)
    pub mod draw_util {
        use super::*;
        // The class itself isn't translated: another class has the same name, in a different package. Its static parts are below.
        // DrawUtil.blitQuad(0): not translated, it calls DrawUtil.blitQuad, which isn't translated.
        // DrawUtil.drawTexQuad(10): not translated, it uses Minecraft's `BufferBuilder`.
        // DrawUtil.blitQuad(4): not translated, it uses Minecraft's `Tesselator`.
        // DrawUtil.drawFramebuffer(2): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
    }

    /// `MathUtil` (net/vulkanmod/render/util/MathUtil.java)
    pub mod math_util {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct MathUtil {
        }
        impl MathUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // MathUtil.clamp(3): not translated, it's overloaded with the same number of parameters.
        // MathUtil.clamp(3): not translated, it's overloaded with the same number of parameters.
        // MathUtil.saturate(1): not translated, it calls MathUtil.clamp, which isn't translated.
        pub fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
            return v0 + (t * (v1 - v0));
        }
        // MathUtil.transformX(4): not translated, JOML's Matrix4f.m00.
        // MathUtil.transformY(4): not translated, JOML's Matrix4f.m01.
        // MathUtil.transformZ(4): not translated, JOML's Matrix4f.m02.
        // MathUtil.packTransformedNorm(5): not translated, it calls `Math` (org.joml).fma.
        // MathUtil.transformNormX(4): not translated, JOML's Matrix3f.m00.
        // MathUtil.transformNormY(4): not translated, JOML's Matrix3f.m01.
        // MathUtil.transformNormZ(4): not translated, JOML's Matrix3f.m02.
    }

    /// `SortUtil` (net/vulkanmod/render/util/SortUtil.java)
    pub mod sort_util {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct SortUtil {
        }
        impl SortUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // SortUtil.mergeSort(2): not translated, it calls SortUtil.mergeSort, which isn't translated.
        // SortUtil.mergeSort(5): not translated, it calls `Arrays` (java.util).copyOf.
        // SortUtil.quickSort(2): not translated, it calls SortUtil.quickSort, which isn't translated.
        // SortUtil.quickSort(4): not translated, it calls SortUtil.insertionSort, which isn't translated.
        // SortUtil.insertionSort(4): not translated, `++` inside an expression.
        pub fn swap_4(x: &mut [i32], mut a: i32, mut b: i32, n: i32) {
            let mut i: i32 = 0;
            while i < n {
                sort_util::swap_3(&mut *x, a, b);
                b += 1;
                i += 1;
                a += 1;
            }
        }
        pub fn swap_3(x: &mut [i32], a: i32, b: i32) {
            let t: i32 = x[a as usize];
            x[a as usize] = x[b as usize];
            x[b as usize] = t;
        }
        // SortUtil.med3(5): not translated, it uses `IntComparator` (it.unimi.dsi.fastutil.ints).
    }

    /// `VBO` (net/vulkanmod/render/VBO.java)
    pub mod vbo {
        use super::*;
        // The class itself isn't translated: it uses VertexBuffer, which isn't translated (it extends `Buffer`). Its static parts are below.
        // VBO.VBO(1): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.upload(1): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.uploadVertexBuffer(2): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.uploadIndexBuffer(1): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.getAutoIndexBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.bind(1): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.draw(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.close(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.getVertexBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.getVertexCount(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.getIndexCount(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.isAutoIndexed(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.getMode(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // VBO.getIndexBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
    }

    /// `CustomVertexFormat` (net/vulkanmod/render/vertex/CustomVertexFormat.java)
    pub mod custom_vertex_format {
        use super::*;
        // ELEMENT_POSITION_INT16: not translated, it uses Minecraft's `VertexFormatElement`.
        // ELEMENT_COLOR_UINT: not translated, it uses Minecraft's `VertexFormatElement`.
        // ELEMENT_UV0_UINT16: not translated, it uses Minecraft's `VertexFormatElement`.
        pub static POSITION_OFFSET: JCell<f32> = JCell::new(4.0f32);
        // COMPRESSED_TERRAIN: not translated, it uses Minecraft's `VertexFormat`.
        // TERRAIN: not translated, it uses Minecraft's `VertexFormat`.
        // NONE: not translated, it uses Minecraft's `VertexFormat`.
        #[derive(Clone, Debug)]
        pub struct CustomVertexFormat {
        }
        impl CustomVertexFormat {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn set_position_offset(position_offset: f32) {
            custom_vertex_format::POSITION_OFFSET.set(position_offset);
        }
        pub fn get_position_offset() -> f32 {
            return custom_vertex_format::POSITION_OFFSET.get();
        }
    }

    /// `I32_SNorm` (net/vulkanmod/render/vertex/format/I32_SNorm.java)
    pub mod i32_s_norm {
        use super::*;
        pub const NORM_INV: f32 = 0.007874016f32;
        #[derive(Clone, Debug)]
        pub struct I32_SNorm {
        }
        impl I32_SNorm {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // I32_SNorm.packNormal(3): not translated, it's overloaded with the same number of parameters.
        // I32_SNorm.packNormal(3): not translated, it's overloaded with the same number of parameters.
        pub fn unpack_x(i: i32) -> f32 {
            return (((i & 0xFF) as i8) as f32) * 0.007874016f32;
        }
        pub fn unpack_y(i: i32) -> f32 {
            return ((((i >> 8) & 0xFF) as i8) as f32) * 0.007874016f32;
        }
        pub fn unpack_z(i: i32) -> f32 {
            return ((((i >> 16) & 0xFF) as i8) as f32) * 0.007874016f32;
        }
    }

    /// `QuadSorter` (net/vulkanmod/render/vertex/QuadSorter.java)
    pub mod quad_sorter {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `VertexFormat`. Its static parts are below.
        // QuadSorter.setQuadSortOrigin(3): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // QuadSorter.getSortState(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // QuadSorter.restoreSortState(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // QuadSorter.setupQuadSortingPoints(3): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // QuadSorter.putSortedQuadIndices(2): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // QuadSorter.putSortedQuadIndices(2): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // QuadSorter.reset(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // QuadSorter.getVertexCount(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // QuadSorter.getIndexCount(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
    }

    /// `QuadSorter.SortState` (net/vulkanmod/render/vertex/QuadSorter.java)
    pub mod quad_sorter_sort_state {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct SortState {
            pub vertex_count: i32,
            pub sorting_points: Option<Vec<JVec3>>,
            pub distances: Option<Vec<f32>>,
            pub sorting_points_indices: Option<Vec<i32>>,
        }
        impl SortState {
            pub fn new(vertex_count: i32, sorting_points: Option<Vec<JVec3>>, distances: Option<Vec<f32>>, sorting_points_indices: Option<Vec<i32>>) -> Self {
                let mut s = Self { vertex_count: vertex_count, sorting_points: sorting_points.clone(), distances: distances.clone(), sorting_points_indices: sorting_points_indices.clone() };
                s.vertex_count = vertex_count;
                s.sorting_points = sorting_points.clone();
                s.distances = distances.clone();
                s.sorting_points_indices = sorting_points_indices.clone();
                s
            }
        }
    }

    /// `TerrainBufferBuilder` (net/vulkanmod/render/vertex/TerrainBufferBuilder.java)
    pub mod terrain_buffer_builder {
        use super::*;
        // The class itself isn't translated: it implements Minecraft's `VertexConsumer`. Its static parts are below.
        // LOGGER: the logger. Log calls become comments.
        // ALLOCATOR: not translated, it uses `MemoryAllocator` (org.lwjgl.system.MemoryUtil).
        // TerrainBufferBuilder.TerrainBufferBuilder(3): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.ensureCapacity(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.ensureCapacity(1): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.resize(1): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.endVertex(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.vertex(8): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.end(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.clear(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.free(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.getBuffer(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.getPtr(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.getVertices(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.getNextElementByte(0): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.addVertex(3): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.setColor(4): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.setColor(1): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.setUv(2): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.setLight(1): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.setNormal(3): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.setLineWidth(1): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.setUv1(2): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
        // TerrainBufferBuilder.setUv2(2): not translated, its class isn't translated (it implements Minecraft's `VertexConsumer`).
    }

    /// `TerrainBuilder` (net/vulkanmod/render/vertex/TerrainBuilder.java)
    pub mod terrain_builder {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `VertexFormat`. Its static parts are below.
        // LOGGER: the logger. Log calls become comments.
        // ALLOCATOR: not translated, it uses `MemoryAllocator` (org.lwjgl.system.MemoryUtil).
        // TerrainBuilder.TerrainBuilder(2): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.getBufferBuilder(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.ensureIndexCapacity(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.resizeIndexBuffer(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.setupQuadSorting(3): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.getSortState(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.restoreSortState(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.setIndexOnly(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.begin(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.setupQuadSortingPoints(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.endDrawing(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.getIndexBuffer(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.ensureDrawing(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.reset(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.clear(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.free(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // TerrainBuilder.setBlockAttributes(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
    }

    /// `TerrainBuilder.DrawState` (net/vulkanmod/render/vertex/TerrainBuilder.java)
    pub mod terrain_builder_draw_state {
        use super::*;
        // The class itself isn't translated: it uses IndexType, which isn't translated (its constants carry values). Its static parts are below.
        // TerrainBuilder.DrawState.DrawState(5): not translated, its class isn't translated (it uses IndexType, which isn't translated (its constants carry values)).
        // TerrainBuilder.DrawState.indexBufferSize(0): not translated, its class isn't translated (it uses IndexType, which isn't translated (its constants carry values)).
        // TerrainBuilder.DrawState.vertexSize(0): not translated, its class isn't translated (it uses IndexType, which isn't translated (its constants carry values)).
        // TerrainBuilder.DrawState.indexCount(0): not translated, its class isn't translated (it uses IndexType, which isn't translated (its constants carry values)).
        // TerrainBuilder.DrawState.indexType(0): not translated, its class isn't translated (it uses IndexType, which isn't translated (its constants carry values)).
        // TerrainBuilder.DrawState.indexOnly(0): not translated, its class isn't translated (it uses IndexType, which isn't translated (its constants carry values)).
        // TerrainBuilder.DrawState.sequentialIndex(0): not translated, its class isn't translated (it uses IndexType, which isn't translated (its constants carry values)).
    }

    /// `TerrainRenderType` (net/vulkanmod/render/vertex/TerrainRenderType.java)
    pub mod terrain_render_type {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // VALUES: not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // COMPACT_RENDER_TYPES: not translated, it uses `EnumSet` (java.util).
        // SEMI_COMPACT_RENDER_TYPES: not translated, it uses `EnumSet` (java.util).
        // remapper: not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // TerrainRenderType.TerrainRenderType(1): not translated, its class isn't translated (its constants carry values).
        // TerrainRenderType.setCutoutUniform(0): not translated, its class isn't translated (its constants carry values).
        // TerrainRenderType.get(1): not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // TerrainRenderType.get(1): not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // TerrainRenderType.get(1): not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
        // TerrainRenderType.getLayer(1): not translated, it uses Minecraft's `ChunkSectionLayer`.
        // TerrainRenderType.updateMapping(0): not translated, a nullable object kept in a changing static.
        // TerrainRenderType.getRemapped(1): not translated, it uses TerrainRenderType, which isn't translated (its constants carry values).
    }

    /// `VertexBuilder` (net/vulkanmod/render/vertex/VertexBuilder.java)
    pub mod vertex_builder {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // VertexBuilder.vertex(9): not translated, it has no body.
        // VertexBuilder.position(4): not translated, it has no body.
        // VertexBuilder.color(2): not translated, it has no body.
        // VertexBuilder.uv(3): not translated, it has no body.
        // VertexBuilder.light(2): not translated, it has no body.
        // VertexBuilder.normal(2): not translated, it has no body.
        // VertexBuilder.getStride(0): not translated, it has no body.
    }

    /// `VertexBuilder.CompressedVertexBuilder` (net/vulkanmod/render/vertex/VertexBuilder.java)
    pub mod vertex_builder_compressed_vertex_builder {
        use super::*;
        // The class itself isn't translated: it implements `VertexBuilder`. Its static parts are below.
        pub const VERTEX_SIZE: i32 = 16;
        pub const POS_CONV_MUL: f32 = 2048.0f32;
        pub const POS_OFFSET: f32 = -4.0f32;
        pub const POS_OFFSET_CONV: f32 = -8192.0f32;
        pub const UV_CONV_MUL: f32 = 32768.0f32;
        // VertexBuilder.CompressedVertexBuilder.vertex(9): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.CompressedVertexBuilder.position(4): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.CompressedVertexBuilder.color(2): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.CompressedVertexBuilder.uv(3): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.CompressedVertexBuilder.light(2): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.CompressedVertexBuilder.normal(2): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.CompressedVertexBuilder.getStride(0): not translated, its class isn't translated (it implements `VertexBuilder`).
    }

    /// `VertexBuilder.DefaultVertexBuilder` (net/vulkanmod/render/vertex/VertexBuilder.java)
    pub mod vertex_builder_default_vertex_builder {
        use super::*;
        // The class itself isn't translated: it implements `VertexBuilder`. Its static parts are below.
        pub const VERTEX_SIZE: i32 = 32;
        // VertexBuilder.DefaultVertexBuilder.vertex(9): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.DefaultVertexBuilder.position(4): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.DefaultVertexBuilder.color(2): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.DefaultVertexBuilder.uv(3): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.DefaultVertexBuilder.light(2): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.DefaultVertexBuilder.normal(2): not translated, its class isn't translated (it implements `VertexBuilder`).
        // VertexBuilder.DefaultVertexBuilder.getStride(0): not translated, its class isn't translated (it implements `VertexBuilder`).
    }

    /// `Device` (net/vulkanmod/vulkan/device/Device.java)
    pub mod device {
        use super::*;
        // The class itself isn't translated: it uses `VkPhysicalDevice` (org.lwjgl.vulkan). Its static parts are below.
        // Device.Device(1): not translated, its class isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // Device.hasExtension(1): not translated, its class isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // Device.getUnsupportedExtensions(1): not translated, its class isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // Device.isDrawIndirectSupported(0): not translated, its class isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // Device.isAMD(0): not translated, its class isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // Device.isNvidia(0): not translated, its class isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // Device.isIntel(0): not translated, its class isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        pub fn decode_vendor(i: i32) -> String {
            return match i {
                4098 | 4130 => "AMD".to_string(),
                4112 => "Imagination Technologies".to_string(),
                4203 => "Apple".to_string(),
                4318 => "Nvidia".to_string(),
                5045 => "ARM".to_string(),
                5348 => "Broadcom".to_string(),
                6880 => "Google".to_string(),
                20803 => "Qualcomm".to_string(),
                32902 => "Intel".to_string(),
                65541 => "Mesa".to_string(),
                _ => "undef".to_string(),
            };
        }
        // Device.decDefVersion(1): not translated, it calls `VK10` (org.lwjgl.vulkan).VK_VERSION_MAJOR.
        // Device.decodeDvrVersion(2): not translated, it calls Device.decDefVersion, which isn't translated.
        // Device.decIntelVersion(1): not translated, it calls `GLFW` (org.lwjgl.glfw).glfwGetPlatform.
        pub fn decode_nvidia(v: i32) -> String {
            return format!("{}.{}.{}.{}", (((v as u32) >> 22) as i32) & 1023, (((v as u32) >> 14) as i32) & 0xFF, (((v as u32) >> 6) as i32) & 0xFF, v & 0xFF);
        }
        // Device.getAvailableExtension(1): not translated, it uses `VkPhysicalDevice` (org.lwjgl.vulkan).
    }

    /// `DeviceManager` (net/vulkanmod/vulkan/device/DeviceManager.java)
    pub mod device_manager {
        use super::*;
        // availableDevices: not translated, it uses Device, which isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // suitableDevices: not translated, it uses Device, which isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // physicalDevice: not translated, it uses `VkPhysicalDevice` (org.lwjgl.vulkan).
        // vkDevice: not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        // device: not translated, it uses Device, which isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // deviceProperties: not translated, it uses `VkPhysicalDeviceProperties` (org.lwjgl.vulkan).
        // memoryProperties: not translated, it uses `VkPhysicalDeviceMemoryProperties` (org.lwjgl.vulkan).
        // surfaceProperties: not translated, it uses SurfaceProperties, which isn't translated (it uses `VkSurfaceCapabilitiesKHR` (org.lwjgl.vulkan)).
        // graphicsQueue: not translated, it uses GraphicsQueue, which isn't translated (it extends `Queue`).
        // presentQueue: not translated, it uses PresentQueue, which isn't translated (it extends `Queue`).
        // transferQueue: not translated, it uses TransferQueue, which isn't translated (it extends `Queue`).
        // computeQueue: not translated, it uses ComputeQueue, which isn't translated (it extends `Queue`).
        #[derive(Clone, Debug)]
        pub struct DeviceManager {
        }
        impl DeviceManager {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // DeviceManager.init(1): not translated, it uses `VkInstance` (org.lwjgl.vulkan).
        // DeviceManager.getAvailableDevices(1): not translated, its body couldn't be read: a labelled statement.
        // DeviceManager.getSuitableDevices(1): not translated, it uses `VkInstance` (org.lwjgl.vulkan).
        // DeviceManager.pickPhysicalDevice(0): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // DeviceManager.autoPickDevice(0): not translated, it uses Device, which isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // DeviceManager.createLogicalDevice(0): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // DeviceManager.getRequiredExtensions(0): not translated, it uses `PointerBuffer` (org.lwjgl).
        // DeviceManager.isDeviceSuitable(1): not translated, it uses Device, which isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
        // DeviceManager.getAvailableExtension(2): not translated, it uses `Buffer` (org.lwjgl.vulkan.VkDeviceQueueCreateInfo).
        // DeviceManager.findDepthFormat(1): not translated, it calls DeviceManager.findSupportedFormat, which isn't translated.
        // DeviceManager.findSupportedFormat(3): not translated, its body couldn't be read: a labelled statement.
        // DeviceManager.getAvailableDevicesInfo(0): not translated, it uses DeviceManager.availableDevices, which isn't translated (it uses Device, which isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan))).
        // DeviceManager.destroy(0): not translated, it uses DeviceManager.graphicsQueue, which isn't translated (it uses GraphicsQueue, which isn't translated (it extends `Queue`)).
        // DeviceManager.getGraphicsQueue(0): not translated, it uses GraphicsQueue, which isn't translated (it extends `Queue`).
        // DeviceManager.getPresentQueue(0): not translated, it uses PresentQueue, which isn't translated (it extends `Queue`).
        // DeviceManager.getTransferQueue(0): not translated, it uses TransferQueue, which isn't translated (it extends `Queue`).
        // DeviceManager.getComputeQueue(0): not translated, it uses ComputeQueue, which isn't translated (it extends `Queue`).
        // DeviceManager.querySurfaceProperties(2): not translated, it uses `VkPhysicalDevice` (org.lwjgl.vulkan).
    }

    /// `DeviceManager.SurfaceProperties` (net/vulkanmod/vulkan/device/DeviceManager.java)
    pub mod device_manager_surface_properties {
        use super::*;
        // The class itself isn't translated: it uses `VkSurfaceCapabilitiesKHR` (org.lwjgl.vulkan). Its static parts are below.
    }

    /// `Drawer` (net/vulkanmod/vulkan/Drawer.java)
    pub mod drawer {
        use super::*;
        // The class itself isn't translated: it uses VertexBuffer, which isn't translated (it extends `Buffer`). Its static parts are below.
        pub const INITIAL_VB_SIZE: i32 = 4000000;
        pub const INITIAL_IB_SIZE: i32 = 1000000;
        pub const INITIAL_UB_SIZE: i32 = 200000;
        // buffers: not translated, it uses `LongBuffer` (java.nio).
        // offsets: not translated, it uses `LongBuffer` (java.nio).
        // pBuffers: not translated, it uses Drawer.buffers, which isn't translated (it uses `LongBuffer` (java.nio)).
        // pOffsets: not translated, it uses Drawer.offsets, which isn't translated (it uses `LongBuffer` (java.nio)).
        // Drawer.setCurrentFrame(1): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.createResources(1): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.resetBuffers(1): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.draw(4): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.draw(5): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.drawIndexed(3): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.drawIndexed(4): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.draw(2): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.bindIndexBuffer(3): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.cleanUpResources(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.getQuadsIndexBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.getLinesIndexBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.getTriangleFanIndexBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.getTriangleStripIndexBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.getDebugLineStripIndexBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.getUniformBuffer(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.getAutoIndexBuffer(2): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        // Drawer.getGpuBuffers(0): not translated, its class isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
    }

    /// `Drawer.GpuBuffers` (net/vulkanmod/vulkan/Drawer.java)
    pub mod drawer_gpu_buffers {
        use super::*;
        // The class itself isn't translated: it uses VkGpuBuffer, which isn't translated (it extends Minecraft's `GpuBuffer`). Its static parts are below.
        // Drawer.GpuBuffers.createBuffers(2): not translated, its class isn't translated (it uses VkGpuBuffer, which isn't translated (it extends Minecraft's `GpuBuffer`)).
        // Drawer.GpuBuffers.getVertexBuffer(0): not translated, its class isn't translated (it uses VkGpuBuffer, which isn't translated (it extends Minecraft's `GpuBuffer`)).
        // Drawer.GpuBuffers.getIndexBuffer(0): not translated, its class isn't translated (it uses VkGpuBuffer, which isn't translated (it extends Minecraft's `GpuBuffer`)).
    }

    /// `Framebuffer` (net/vulkanmod/vulkan/framebuffer/Framebuffer.java)
    pub mod framebuffer {
        use super::*;
        // The class itself isn't translated: it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects). Its static parts are below.
        pub const DEFAULT_FORMAT: i32 = 37;
        // Framebuffer.Framebuffer(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.Framebuffer(1): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.createImages(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.resize(2): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.createFramebuffer(1): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.beginRenderPass(3): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getFramebufferId(1): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.viewport(1): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.scissor(1): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.cleanUp(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.cleanUp(1): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.setLevel(1): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getDepthImageView(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getDepthAttachment(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getColorAttachment(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getColorAttachmentView(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getWidth(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getHeight(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getFormat(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.getDepthFormat(0): not translated, its class isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.builder(4): not translated, it creates a Builder with a constructor that isn't translated.
        // Framebuffer.builder(2): not translated, it stores a second reference to an object (Rust would need it copied or shared).
    }

    /// `Framebuffer.Builder` (net/vulkanmod/vulkan/framebuffer/Framebuffer.java)
    pub mod framebuffer_builder {
        use super::*;
        // The class itself isn't translated: it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)). Its static parts are below.
        // Framebuffer.Builder.Builder(4): not translated, `null` (Rust has no null: this needs an Option).
        // Framebuffer.Builder.Builder(5): not translated, it uses `Object`.
        // Framebuffer.Builder.Builder(2): not translated, its class isn't translated (it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints))).
        // Framebuffer.Builder.build(0): not translated, it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Framebuffer.Builder.setLevel(1): not translated, `this` used as a value.
        // Framebuffer.Builder.setFormat(1): not translated, `this` used as a value.
        // Framebuffer.Builder.setLinearFiltering(1): not translated, `this` used as a value.
        // Framebuffer.Builder.setDepthLinearFiltering(1): not translated, `this` used as a value.
    }

    /// `RenderPass` (net/vulkanmod/vulkan/framebuffer/RenderPass.java)
    pub mod render_pass {
        use super::*;
        // The class itself isn't translated: it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)). Its static parts are below.
        // RenderPass.RenderPass(3): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.createRenderPass(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.beginRenderPass(3): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.endRenderPass(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.beginDynamicRendering(2): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.getFramebuffer(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.setFramebuffer(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.cleanUp(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.getId(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.builder(1): not translated, it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
    }

    /// `RenderPass.AttachmentInfo` (net/vulkanmod/vulkan/framebuffer/RenderPass.java)
    pub mod render_pass_attachment_info {
        use super::*;
        // The class itself isn't translated: it uses Type, which isn't translated (its constants carry values). Its static parts are below.
        // RenderPass.AttachmentInfo.AttachmentInfo(2): not translated, its class isn't translated (it uses Type, which isn't translated (its constants carry values)).
        // RenderPass.AttachmentInfo.setOps(2): not translated, its class isn't translated (it uses Type, which isn't translated (its constants carry values)).
        // RenderPass.AttachmentInfo.setLoadOp(1): not translated, its class isn't translated (it uses Type, which isn't translated (its constants carry values)).
        // RenderPass.AttachmentInfo.setFinalLayout(1): not translated, its class isn't translated (it uses Type, which isn't translated (its constants carry values)).
    }

    /// `RenderPass.AttachmentInfo.Type` (net/vulkanmod/vulkan/framebuffer/RenderPass.java)
    pub mod render_pass_attachment_info_type {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // RenderPass.AttachmentInfo.Type.Type(1): not translated, its class isn't translated (its constants carry values).
    }

    /// `RenderPass.Builder` (net/vulkanmod/vulkan/framebuffer/RenderPass.java)
    pub mod render_pass_builder {
        use super::*;
        // The class itself isn't translated: it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)). Its static parts are below.
        // RenderPass.Builder.Builder(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.Builder.build(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.Builder.setLoadOp(1): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.Builder.getColorAttachmentInfo(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
        // RenderPass.Builder.getDepthAttachmentInfo(0): not translated, its class isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))).
    }

    /// `SwapChain` (net/vulkanmod/vulkan/framebuffer/SwapChain.java)
    pub mod swap_chain {
        use super::*;
        // The class itself isn't translated: it extends `Framebuffer`. Its static parts are below.
        // defUncappedMode: not translated, it calls SwapChain.checkPresentMode, which isn't translated.
        // SwapChain.SwapChain(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.recreate(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.createSwapChain(0): not translated, its body couldn't be read: a labelled statement.
        // SwapChain.createFramebuffers(1): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.createDepthResources(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getFramebufferId(1): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.cleanUp(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getFormat(1): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getPresentMode(1): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getDisplayModeString(1): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getExtent(1): not translated, it uses `VkExtent2D` (org.lwjgl.vulkan).
        // SwapChain.checkPresentMode(1): not translated, its body couldn't be read: a labelled statement.
        // SwapChain.getId(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getImages(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getImageId(1): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getExtent(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getColorAttachment(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getColorAttachmentView(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.hasImages(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.isAcquired(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.setAcquired(1): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.isVsync(0): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.setVsync(1): not translated, its class isn't translated (it extends `Framebuffer`).
        // SwapChain.getImagesNum(0): not translated, its class isn't translated (it extends `Framebuffer`).
    }

    /// `Buffer` (net/vulkanmod/vulkan/memory/buffer/Buffer.java)
    pub mod buffer {
        use super::*;
        // The class itself isn't translated: it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)). Its static parts are below.
        // Buffer.Buffer(3): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.createBuffer(1): not translated, `this` used as a value.
        // Buffer.resizeBuffer(1): not translated, it's overloaded with the same number of parameters.
        // Buffer.copyBuffer(2): not translated, it uses `ByteBuffer` (java.nio).
        // Buffer.copyBuffer(3): not translated, it uses `ByteBuffer` (java.nio).
        // Buffer.scheduleFree(0): not translated, it's overloaded with the same number of parameters.
        // Buffer.reset(0): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.getAllocation(0): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.getUsedBytes(0): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.getOffset(0): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.getId(0): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.getBufferSize(0): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.getDataPtr(0): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.setBufferSize(1): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.setId(1): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.setAllocation(1): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
        // Buffer.getBufferInfo(0): not translated, its class isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))).
    }

    /// `Buffer.BufferInfo` (net/vulkanmod/vulkan/memory/buffer/Buffer.java)
    pub mod buffer_buffer_info {
        use super::*;
        #[derive(Clone, Debug, PartialEq)]
        pub struct BufferInfo {
            pub id: i64,
            pub allocation: i64,
            pub buffer_size: i64,
            pub r#type: memory_type_type::Type,
        }
        impl BufferInfo {
            pub fn new(id: i64, allocation: i64, buffer_size: i64, r#type: memory_type_type::Type) -> Self {
                let mut s = Self { id: id, allocation: allocation, buffer_size: buffer_size, r#type: r#type };
                s.id = id;
                s.allocation = allocation;
                s.buffer_size = buffer_size;
                s.r#type = r#type;
                s
            }
            pub fn id(&self) -> i64 {
                return self.id;
            }
            pub fn allocation(&self) -> i64 {
                return self.allocation;
            }
            pub fn buffer_size(&self) -> i64 {
                return self.buffer_size;
            }
            pub fn r#type(&self) -> memory_type_type::Type {
                return self.r#type;
            }
        }
    }

    /// `BufferSlice` (net/vulkanmod/vulkan/memory/buffer/BufferSlice.java)
    pub mod buffer_slice {
        use super::*;
        // The class itself isn't translated: it uses Buffer, which isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan))). Its static parts are below.
        // BufferSlice.set(3): not translated, it stores a second reference to an object (Rust would need it copied or shared).
        // BufferSlice.getBuffer(0): not translated, its class isn't translated (it uses Buffer, which isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)))).
        // BufferSlice.getOffset(0): not translated, its class isn't translated (it uses Buffer, which isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)))).
        // BufferSlice.getSize(0): not translated, its class isn't translated (it uses Buffer, which isn't translated (it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)))).
    }

    /// `IndexBuffer` (net/vulkanmod/vulkan/memory/buffer/IndexBuffer.java)
    pub mod index_buffer {
        use super::*;
        // The class itself isn't translated: it extends `Buffer`. Its static parts are below.
        // IndexBuffer.IndexBuffer(2): not translated, its class isn't translated (it extends `Buffer`).
        // IndexBuffer.IndexBuffer(3): not translated, its class isn't translated (it extends `Buffer`).
    }

    /// `IndexBuffer.IndexType` (net/vulkanmod/vulkan/memory/buffer/IndexBuffer.java)
    pub mod index_buffer_index_type {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // IndexBuffer.IndexType.IndexType(2): not translated, its class isn't translated (its constants carry values).
    }

    /// `AutoIndexBuffer` (net/vulkanmod/vulkan/memory/buffer/index/AutoIndexBuffer.java)
    pub mod auto_index_buffer {
        use super::*;
        // The class itself isn't translated: it uses DrawType, which isn't translated (its constants carry values). Its static parts are below.
        pub const U16_MAX_VERTEX_COUNT: i32 = 65536;
        pub const QUAD_U16_MAX_INDEX_COUNT: i32 = 98304;
        // AutoIndexBuffer.AutoIndexBuffer(2): not translated, its class isn't translated (it uses DrawType, which isn't translated (its constants carry values)).
        // AutoIndexBuffer.createIndexBuffer(1): not translated, its class isn't translated (it uses DrawType, which isn't translated (its constants carry values)).
        // AutoIndexBuffer.checkCapacity(1): not translated, its class isn't translated (it uses DrawType, which isn't translated (its constants carry values)).
        // AutoIndexBuffer.getIndexBuffer(0): not translated, its class isn't translated (it uses DrawType, which isn't translated (its constants carry values)).
        // AutoIndexBuffer.freeBuffer(0): not translated, its class isn't translated (it uses DrawType, which isn't translated (its constants carry values)).
        // AutoIndexBuffer.getIndexCount(1): not translated, its class isn't translated (it uses DrawType, which isn't translated (its constants carry values)).
        // AutoIndexBuffer.getIndexCount(2): not translated, it uses DrawType, which isn't translated (its constants carry values).
        // AutoIndexBuffer.maxVertexCount(2): not translated, it uses DrawType, which isn't translated (its constants carry values).
        // AutoIndexBuffer.genQuadIndices(1): not translated, it uses `ByteBuffer` (java.nio).
        // AutoIndexBuffer.genIntQuadIndices(1): not translated, it uses `ByteBuffer` (java.nio).
        // AutoIndexBuffer.genLinesIndices(1): not translated, it uses `ByteBuffer` (java.nio).
        // AutoIndexBuffer.genIntLinesIndices(1): not translated, it uses `ByteBuffer` (java.nio).
        // AutoIndexBuffer.genTriangleFanIndices(1): not translated, it uses `ByteBuffer` (java.nio).
        // AutoIndexBuffer.genTriangleStripIndices(1): not translated, it uses `ByteBuffer` (java.nio).
        // AutoIndexBuffer.genDebugLineStripIndices(1): not translated, it uses `ByteBuffer` (java.nio).
        pub fn round_up_to_divisible(n: i32, d: i32) -> i32 {
            return (((n + d) - 1) / d) * d;
        }
    }

    /// `AutoIndexBuffer.DrawType` (net/vulkanmod/vulkan/memory/buffer/index/AutoIndexBuffer.java)
    pub mod auto_index_buffer_draw_type {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // AutoIndexBuffer.DrawType.DrawType(1): not translated, its class isn't translated (its constants carry values).
        pub fn get_quad_index_count(vertex_count: i32) -> i32 {
            return (vertex_count * 3) / 2;
        }
        pub fn get_triangle_strip_index_count(vertex_count: i32) -> i32 {
            return (vertex_count - 2) * 3;
        }
    }

    /// `IndirectBuffer` (net/vulkanmod/vulkan/memory/buffer/IndirectBuffer.java)
    pub mod indirect_buffer {
        use super::*;
        // The class itself isn't translated: it extends `Buffer`. Its static parts are below.
        // IndirectBuffer.IndirectBuffer(2): not translated, its class isn't translated (it extends `Buffer`).
        // IndirectBuffer.recordCopyCmd(1): not translated, its class isn't translated (it extends `Buffer`).
        // IndirectBuffer.submitUploads(0): not translated, its class isn't translated (it extends `Buffer`).
    }

    /// `StagingBuffer` (net/vulkanmod/vulkan/memory/buffer/StagingBuffer.java)
    pub mod staging_buffer {
        use super::*;
        // The class itself isn't translated: it extends `Buffer`. Its static parts are below.
        pub const DEFAULT_SIZE: i64 = 67108864i64;
        // StagingBuffer.StagingBuffer(0): not translated, its class isn't translated (it extends `Buffer`).
        // StagingBuffer.StagingBuffer(1): not translated, its class isn't translated (it extends `Buffer`).
        // StagingBuffer.copyBuffer(2): not translated, its class isn't translated (it extends `Buffer`).
        // StagingBuffer.copyBuffer(2): not translated, its class isn't translated (it extends `Buffer`).
        // StagingBuffer.align(1): not translated, its class isn't translated (it extends `Buffer`).
        // StagingBuffer.submitUploads(0): not translated, its class isn't translated (it extends `Buffer`).
    }

    /// `StagingBuffers` (net/vulkanmod/vulkan/memory/buffer/StagingBuffers.java)
    pub mod staging_buffers {
        use super::*;
        // The class itself isn't translated: it uses `Stack` (it.unimi.dsi.fastutil). Its static parts are below.
        // StagingBuffers.updateFrameCount(1): not translated, its class isn't translated (it uses `Stack` (it.unimi.dsi.fastutil)).
        // StagingBuffers.getStagingBuffer(0): not translated, its class isn't translated (it uses `Stack` (it.unimi.dsi.fastutil)).
        // StagingBuffers.beginFrame(1): not translated, its class isn't translated (it uses `Stack` (it.unimi.dsi.fastutil)).
        // StagingBuffers.endFrame(1): not translated, its class isn't translated (it uses `Stack` (it.unimi.dsi.fastutil)).
        // StagingBuffers.free(0): not translated, its class isn't translated (it uses `Stack` (it.unimi.dsi.fastutil)).
    }

    /// `UniformBuffer` (net/vulkanmod/vulkan/memory/buffer/UniformBuffer.java)
    pub mod uniform_buffer {
        use super::*;
        // The class itself isn't translated: it extends `Buffer`. Its static parts are below.
        // MIN_OFFSET_ALIGNMENT: not translated, it uses DeviceManager.deviceProperties, which isn't translated (it uses `VkPhysicalDeviceProperties` (org.lwjgl.vulkan)).
        // UniformBuffer.getAlignedSize(1): not translated, it uses UniformBuffer.MIN_OFFSET_ALIGNMENT, which isn't translated (it uses DeviceManager.deviceProperties, which isn't translated (it uses `VkPhysicalDeviceProperties` (org.lwjgl.vulkan))).
        // UniformBuffer.UniformBuffer(2): not translated, its class isn't translated (it extends `Buffer`).
        // UniformBuffer.checkCapacity(1): not translated, its class isn't translated (it extends `Buffer`).
        // UniformBuffer.updateOffset(1): not translated, its class isn't translated (it extends `Buffer`).
        // UniformBuffer.getPointer(0): not translated, its class isn't translated (it extends `Buffer`).
    }

    /// `VertexBuffer` (net/vulkanmod/vulkan/memory/buffer/VertexBuffer.java)
    pub mod vertex_buffer {
        use super::*;
        // The class itself isn't translated: it extends `Buffer`. Its static parts are below.
        // VertexBuffer.VertexBuffer(1): not translated, its class isn't translated (it extends `Buffer`).
        // VertexBuffer.VertexBuffer(2): not translated, its class isn't translated (it extends `Buffer`).
    }

    /// `MemoryManager` (net/vulkanmod/vulkan/memory/MemoryManager.java)
    pub mod memory_manager {
        use super::*;
        // The class itself isn't translated: it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects). Its static parts are below.
        pub const DEBUG: bool = false;
        pub const BYTES_IN_MB: i64 = 1048576i64;
        // INSTANCE: not translated, it uses MemoryManager, which isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // ALLOCATOR: not translated, its value reads other changing statics.
        // buffers: not translated, it uses `Long2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.longs).
        // images: not translated, it uses `Long2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.longs).
        pub static FRAMES: JCell<i32> = JCell::new(0i32);
        pub static DEVICE_MEMORY: JCell<i64> = JCell::new(0i64);
        pub static NATIVE_MEMORY: JCell<i64> = JCell::new(0i64);
        // MemoryManager.getInstance(0): not translated, it uses MemoryManager, which isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.createInstance(1): not translated, it uses MemoryManager, which isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.MemoryManager(0): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.initFrame(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.setCurrentFrame(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.freeAllBuffers(0): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.createBuffer(5): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.createBuffer(4): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.createImage(11): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.addImage(1): not translated, it uses MemoryManager.images, which isn't translated (it uses `Long2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.longs)).
        // MemoryManager.MapAndCopy(2): not translated, it uses `PointerBuffer` (org.lwjgl).
        // MemoryManager.Map(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.freeBuffer(2): not translated, it uses MemoryManager.ALLOCATOR, which isn't translated (its value reads other changing statics).
        // MemoryManager.freeBuffer(1): not translated, it uses MemoryManager.ALLOCATOR, which isn't translated (its value reads other changing statics).
        // MemoryManager.freeImage(2): not translated, it uses MemoryManager.ALLOCATOR, which isn't translated (its value reads other changing statics).
        // MemoryManager.addToFreeable(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.addToFreeable(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.addFrameOp(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.doFrameOps(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.freeBuffers(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.freeImages(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.checkBuffer(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.freeSegments(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.addToFreeSegment(2): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.getNativeMemoryMB(0): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.getAllocatedDeviceMemoryMB(0): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.getDeviceMemoryMB(0): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.bytesInMb(1): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
        // MemoryManager.getHeapStats(0): not translated, its class isn't translated (it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects)).
    }

    /// `MemoryType` (net/vulkanmod/vulkan/memory/MemoryType.java)
    pub mod memory_type {
        use super::*;
        // The class itself isn't translated: it uses `VkMemoryType` (org.lwjgl.vulkan). Its static parts are below.
        // MemoryType.MemoryType(3): not translated, its class isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)).
        // MemoryType.createBuffer(2): not translated, it has no body.
        // MemoryType.copyToBuffer(5): not translated, it has no body.
        // MemoryType.copyFromBuffer(3): not translated, it has no body.
        // MemoryType.mappable(0): not translated, it has no body.
        // MemoryType.getType(0): not translated, its class isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)).
    }

    /// `MemoryType.Type` (net/vulkanmod/vulkan/memory/MemoryType.java)
    pub mod memory_type_type {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum Type {
            DeviceLocal,
            HostLocal,
        }
        impl Type {
            pub const VALUES: [Type; 2] = [Type::DeviceLocal, Type::HostLocal, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    Type::DeviceLocal => "DEVICE_LOCAL",
                    Type::HostLocal => "HOST_LOCAL",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "DEVICE_LOCAL" => Type::DeviceLocal,
                    "HOST_LOCAL" => Type::HostLocal,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `MemoryTypes` (net/vulkanmod/vulkan/memory/MemoryTypes.java)
    pub mod memory_types {
        use super::*;
        // GPU_MEM: not translated, it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)).
        // HOST_MEM: not translated, it uses MemoryType, which isn't translated (it uses `VkMemoryType` (org.lwjgl.vulkan)).
        #[derive(Clone, Debug)]
        pub struct MemoryTypes {
        }
        impl MemoryTypes {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // MemoryTypes.createMemoryTypes(0): not translated, it uses DeviceManager.memoryProperties, which isn't translated (it uses `VkPhysicalDeviceMemoryProperties` (org.lwjgl.vulkan)).
    }

    /// `MemoryTypes.DeviceLocalMemory` (net/vulkanmod/vulkan/memory/MemoryTypes.java)
    pub mod memory_types_device_local_memory {
        use super::*;
        // The class itself isn't translated: it extends `MemoryType`. Its static parts are below.
        // MemoryTypes.DeviceLocalMemory.DeviceLocalMemory(2): not translated, its class isn't translated (it extends `MemoryType`).
        // MemoryTypes.DeviceLocalMemory.createBuffer(2): not translated, its class isn't translated (it extends `MemoryType`).
        // MemoryTypes.DeviceLocalMemory.copyToBuffer(5): not translated, its class isn't translated (it extends `MemoryType`).
        // MemoryTypes.DeviceLocalMemory.copyFromBuffer(3): not translated, its class isn't translated (it extends `MemoryType`).
        // MemoryTypes.DeviceLocalMemory.copyBuffer(2): not translated, its class isn't translated (it extends `MemoryType`).
        // MemoryTypes.DeviceLocalMemory.mappable(0): not translated, its class isn't translated (it extends `MemoryType`).
    }

    /// `MemoryTypes.DeviceMappableMemory` (net/vulkanmod/vulkan/memory/MemoryTypes.java)
    pub mod memory_types_device_mappable_memory {
        use super::*;
        // The class itself isn't translated: it extends `MappableMemory`. Its static parts are below.
        // MemoryTypes.DeviceMappableMemory.DeviceMappableMemory(2): not translated, its class isn't translated (it extends `MappableMemory`).
        // MemoryTypes.DeviceMappableMemory.createBuffer(2): not translated, its class isn't translated (it extends `MappableMemory`).
    }

    /// `MemoryTypes.HostCoherentMemory` (net/vulkanmod/vulkan/memory/MemoryTypes.java)
    pub mod memory_types_host_coherent_memory {
        use super::*;
        // The class itself isn't translated: it extends `MappableMemory`. Its static parts are below.
        // MemoryTypes.HostCoherentMemory.HostCoherentMemory(2): not translated, its class isn't translated (it extends `MappableMemory`).
        // MemoryTypes.HostCoherentMemory.createBuffer(2): not translated, its class isn't translated (it extends `MappableMemory`).
    }

    /// `MemoryTypes.HostLocalFallbackMemory` (net/vulkanmod/vulkan/memory/MemoryTypes.java)
    pub mod memory_types_host_local_fallback_memory {
        use super::*;
        // The class itself isn't translated: it extends `MappableMemory`. Its static parts are below.
        // MemoryTypes.HostLocalFallbackMemory.HostLocalFallbackMemory(2): not translated, its class isn't translated (it extends `MappableMemory`).
        // MemoryTypes.HostLocalFallbackMemory.createBuffer(2): not translated, its class isn't translated (it extends `MappableMemory`).
    }

    /// `MemoryTypes.MappableMemory` (net/vulkanmod/vulkan/memory/MemoryTypes.java)
    pub mod memory_types_mappable_memory {
        use super::*;
        // The class itself isn't translated: it extends `MemoryType`. Its static parts are below.
        // MemoryTypes.MappableMemory.MappableMemory(3): not translated, its class isn't translated (it extends `MemoryType`).
        // MemoryTypes.MappableMemory.copyToBuffer(5): not translated, its class isn't translated (it extends `MemoryType`).
        // MemoryTypes.MappableMemory.copyFromBuffer(3): not translated, its class isn't translated (it extends `MemoryType`).
        // MemoryTypes.MappableMemory.mappable(0): not translated, its class isn't translated (it extends `MemoryType`).
    }

    /// `DefaultMainPass` (net/vulkanmod/vulkan/pass/DefaultMainPass.java)
    pub mod default_main_pass {
        use super::*;
        // The class itself isn't translated: it implements `MainPass`. Its static parts are below.
        // DefaultMainPass.create(0): not translated, it uses DefaultMainPass, which isn't translated (it implements `MainPass`).
        // DefaultMainPass.DefaultMainPass(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.createResources(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.createRenderPasses(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.begin(2): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.end(1): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.cleanUp(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.onResize(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.rebindMainTarget(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.bindAsTexture(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.getMainFramebuffer(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.getColorAttachment(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.getColorAttachmentView(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.getDepthAttachment(0): not translated, its class isn't translated (it implements `MainPass`).
        // DefaultMainPass.createAttachmentTextures(0): not translated, its class isn't translated (it implements `MainPass`).
    }

    /// `MainPass` (net/vulkanmod/vulkan/pass/MainPass.java)
    pub mod main_pass {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // MainPass.begin(2): not translated, it has no body.
        // MainPass.end(1): not translated, it has no body.
        // MainPass.cleanUp(0): not translated, it has no body.
        // MainPass.onResize(0): not translated, it has no body.
        // MainPass.mainTargetBindWrite(0): not translated, its class isn't translated (it's an interface).
        // MainPass.mainTargetUnbindWrite(0): not translated, its class isn't translated (it's an interface).
        // MainPass.rebindMainTarget(0): not translated, its class isn't translated (it's an interface).
        // MainPass.bindAsTexture(0): not translated, its class isn't translated (it's an interface).
        // MainPass.getMainFramebuffer(0): not translated, its class isn't translated (it's an interface).
        // MainPass.getColorAttachment(0): not translated, its class isn't translated (it's an interface).
        // MainPass.getColorAttachmentView(0): not translated, its class isn't translated (it's an interface).
        // MainPass.getDepthAttachment(0): not translated, its class isn't translated (it's an interface).
    }

    /// `CommandPool` (net/vulkanmod/vulkan/queue/CommandPool.java)
    pub mod command_pool {
        use super::*;
        // The class itself isn't translated: it uses `Queue`. Its static parts are below.
        // CommandPool.CommandPool(1): not translated, its class isn't translated (it uses `Queue`).
        // CommandPool.createCommandPool(1): not translated, its class isn't translated (it uses `Queue`).
        // CommandPool.getCommandBuffer(0): not translated, its class isn't translated (it uses `Queue`).
        // CommandPool.getCommandBuffer(1): not translated, its class isn't translated (it uses `Queue`).
        // CommandPool.allocateCommandBuffers(1): not translated, its class isn't translated (it uses `Queue`).
        // CommandPool.addToAvailable(1): not translated, its class isn't translated (it uses `Queue`).
        // CommandPool.cleanUp(0): not translated, its class isn't translated (it uses `Queue`).
        // CommandPool.getId(0): not translated, its class isn't translated (it uses `Queue`).
    }

    /// `CommandPool.CommandBuffer` (net/vulkanmod/vulkan/queue/CommandPool.java)
    pub mod command_pool_command_buffer {
        use super::*;
        // The class itself isn't translated: it uses CommandPool, which isn't translated (it uses `Queue`). Its static parts are below.
        // CommandPool.CommandBuffer.CommandBuffer(4): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // CommandPool.CommandBuffer.getHandle(0): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // CommandPool.CommandBuffer.getFence(0): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // CommandPool.CommandBuffer.getSemaphore(0): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // CommandPool.CommandBuffer.isSubmitted(0): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // CommandPool.CommandBuffer.isRecording(0): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // CommandPool.CommandBuffer.begin(1): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // CommandPool.CommandBuffer.submitCommands(3): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // CommandPool.CommandBuffer.reset(0): not translated, its class isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
    }

    /// `ComputeQueue` (net/vulkanmod/vulkan/queue/ComputeQueue.java)
    pub mod compute_queue {
        use super::*;
        // The class itself isn't translated: it extends `Queue`. Its static parts are below.
        // ComputeQueue.ComputeQueue(2): not translated, its class isn't translated (it extends `Queue`).
    }

    /// `GraphicsQueue` (net/vulkanmod/vulkan/queue/GraphicsQueue.java)
    pub mod graphics_queue {
        use super::*;
        // The class itself isn't translated: it extends `Queue`. Its static parts are below.
        // INSTANCE: not translated, it uses GraphicsQueue, which isn't translated (it extends `Queue`).
        // currentCmdBuffer: not translated, it uses CommandBuffer, which isn't translated (it uses CommandPool, which isn't translated (it uses `Queue`)).
        // GraphicsQueue.GraphicsQueue(2): not translated, its class isn't translated (it extends `Queue`).
        // GraphicsQueue.startRecording(0): not translated, its class isn't translated (it extends `Queue`).
        // GraphicsQueue.endRecordingAndSubmit(0): not translated, its class isn't translated (it extends `Queue`).
        // GraphicsQueue.getCommandBuffer(0): not translated, its class isn't translated (it extends `Queue`).
        // GraphicsQueue.endIfNeeded(1): not translated, its class isn't translated (it extends `Queue`).
    }

    /// `PresentQueue` (net/vulkanmod/vulkan/queue/PresentQueue.java)
    pub mod present_queue {
        use super::*;
        // The class itself isn't translated: it extends `Queue`. Its static parts are below.
        // PresentQueue.PresentQueue(2): not translated, its class isn't translated (it extends `Queue`).
    }

    /// `Queue` (net/vulkanmod/vulkan/queue/Queue.java)
    pub mod queue {
        use super::*;
        // The class itself isn't translated: it uses `VkQueue` (org.lwjgl.vulkan). Its static parts are below.
        // device: not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        pub static QUEUE_FAMILY_INDICES: LazyLock<JCell<Option<queue_queue_family_indices::QueueFamilyIndices>>> = LazyLock::new(|| JCell::new(None));
        // Queue.beginCommands(0): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.Queue(2): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.Queue(3): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.submitCommands(1): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.submitCommands(2): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.vkQueue(0): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.cleanUp(0): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.waitIdle(0): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.getCommandPool(0): not translated, its class isn't translated (it uses `VkQueue` (org.lwjgl.vulkan)).
        // Queue.getQueueFamilies(0): not translated, it uses Queue.device, which isn't translated (it uses `VkDevice` (org.lwjgl.vulkan)).
        // Queue.findQueueFamilies(1): not translated, it uses `VkPhysicalDevice` (org.lwjgl.vulkan).
    }

    /// `Queue.Family` (net/vulkanmod/vulkan/queue/Queue.java)
    pub mod queue_family {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum Family {
            Graphics,
            Transfer,
            Compute,
        }
        impl Family {
            pub const VALUES: [Family; 3] = [Family::Graphics, Family::Transfer, Family::Compute, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    Family::Graphics => "Graphics",
                    Family::Transfer => "Transfer",
                    Family::Compute => "Compute",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "Graphics" => Family::Graphics,
                    "Transfer" => Family::Transfer,
                    "Compute" => Family::Compute,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `Queue.QueueFamilyIndices` (net/vulkanmod/vulkan/queue/Queue.java)
    pub mod queue_queue_family_indices {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct QueueFamilyIndices {
            pub graphics_family: i32,
            pub present_family: i32,
            pub transfer_family: i32,
            pub compute_family: i32,
        }
        impl QueueFamilyIndices {
            pub fn new() -> Self {
                let mut s = Self { graphics_family: -1, present_family: -1, transfer_family: -1, compute_family: -1 };
                s
            }
            pub fn is_complete(&self) -> bool {
                return (((self.graphics_family != (-1)) && (self.present_family != (-1))) && (self.transfer_family != (-1))) && (self.compute_family != (-1));
            }
            pub fn is_suitable(&self) -> bool {
                return (self.graphics_family != (-1)) && (self.present_family != (-1));
            }
            // Queue.QueueFamilyIndices.unique(0): not translated, it calls `IntStream` (java.util.stream).of.
            pub fn array(&self) -> Vec<i32> {
                return vec![self.graphics_family, self.present_family];
            }
        }
    }

    /// `TransferQueue` (net/vulkanmod/vulkan/queue/TransferQueue.java)
    pub mod transfer_queue {
        use super::*;
        // The class itself isn't translated: it extends `Queue`. Its static parts are below.
        // DEVICE: not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        // TransferQueue.TransferQueue(2): not translated, its class isn't translated (it extends `Queue`).
        // TransferQueue.copyBufferCmd(5): not translated, its class isn't translated (it extends `Queue`).
        // TransferQueue.uploadBufferImmediate(5): not translated, its class isn't translated (it extends `Queue`).
        // TransferQueue.uploadBufferCmd(6): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
    }

    /// `Renderer` (net/vulkanmod/vulkan/Renderer.java)
    pub mod renderer {
        use super::*;
        // The class itself isn't translated: a Map or Set keyed by a Pipeline. Its static parts are below.
        // INSTANCE: not translated, it uses Renderer, which isn't translated (a Map or Set keyed by a Pipeline).
        // device: not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        pub static SWAP_CHAIN_UPDATE: JCell<bool> = JCell::new(false);
        pub static CURRENT_FRAME: JCell<i32> = JCell::new(0);
        pub static IMAGE_INDEX: JCell<i32> = JCell::new(0);
        pub static LAST_RESET: JCell<i32> = JCell::new(-1);
        // Renderer.initRenderer(0): not translated, it uses Renderer, which isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getInstance(0): not translated, it uses Renderer, which isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getDrawer(0): not translated, it uses Drawer, which isn't translated (it uses VertexBuffer, which isn't translated (it extends `Buffer`)).
        pub fn get_current_frame() -> i32 {
            return renderer::CURRENT_FRAME.get();
        }
        pub fn get_current_image() -> i32 {
            return renderer::IMAGE_INDEX.get();
        }
        // Renderer.Renderer(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.setLineWidth(1): not translated, it uses Renderer, which isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.init(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.allocateCommandBuffers(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.createSyncObjects(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.preInitFrame(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.beginFrame(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.beginMainRenderPass(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.endFrame(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.submitFrame(0): not translated, its body couldn't be read: a labelled statement.
        // Renderer.flushCmds(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.submitUploads(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.endRenderPass(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.endRenderPass(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.beginRenderPass(2): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.addUsedPipeline(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.removeUsedPipeline(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.waitFences(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.resetDescriptors(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.waitForSwapChain(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.recreateSwapChain(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.cleanUpResources(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.destroySyncObjects(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.addOnResizeCallback(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.bindGraphicsPipeline(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.uploadAndBindUBOs(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.pushConstants(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getBoundPipeline(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.setBoundFramebuffer(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getBoundFramebuffer(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.setBoundRenderPass(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getBoundRenderPass(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.setMainPass(1): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getMainPass(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getSwapChain(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getTransferCb(0): not translated, its class isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.resetDynamicState(1): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.setDepthBias(2): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.clearAttachments(1): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.clearAttachments(2): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.clearAttachments(3): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.clearAttachments(5): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.clearAttachments(4): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.clearAttachments(6): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.setInvertedViewport(4): not translated, it calls Renderer.setViewportState, which isn't translated.
        // Renderer.resetViewport(0): not translated, it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)).
        // Renderer.setViewportState(4): not translated, it calls Minecraft's `GlStateManager`._viewport.
        // Renderer.setViewport(4): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Renderer.setViewport(5): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Renderer.setScissor(4): not translated, it uses Renderer, which isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.resetScissor(0): not translated, it uses Renderer, which isn't translated (a Map or Set keyed by a Pipeline).
        pub fn push_debug_section(s: &str) {
        }
        pub fn pop_debug_section() {
        }
        pub fn pop_push_debug_section(s: &str) {
            renderer::pop_debug_section();
            renderer::push_debug_section(&s);
        }
        // Renderer.getFramesNum(0): not translated, it uses Renderer, which isn't translated (a Map or Set keyed by a Pipeline).
        // Renderer.getCommandBuffer(0): not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        // Renderer.isRecording(0): not translated, it uses Renderer, which isn't translated (a Map or Set keyed by a Pipeline).
        pub fn schedule_swap_chain_update() {
            renderer::SWAP_CHAIN_UPDATE.set(true);
        }
    }

    /// `Attribute` (net/vulkanmod/vulkan/shader/converter/Attribute.java)
    pub mod attribute {
        use super::*;
        // The class itself isn't translated: it implements `Node`. Its static parts are below.
        // Attribute.Attribute(3): not translated, its class isn't translated (it implements `Node`).
        // Attribute.setLocation(1): not translated, its class isn't translated (it implements `Node`).
        // Attribute.getStringValue(0): not translated, its class isn't translated (it implements `Node`).
    }

    /// `GLSLParser` (net/vulkanmod/vulkan/shader/converter/GLSLParser.java)
    pub mod glsl_parser {
        use super::*;
        // The class itself isn't translated: it uses `LinkedList` (java.util). Its static parts are below.
        // GLSLParser.setVertexFormat(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.parse(2): not translated, its body couldn't be read: a labelled statement.
        // GLSLParser.parseVersion(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.parsePreprocessor(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.parseUniform(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.parseStorageUniform(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.parseSampler(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.parseUniformBlock(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.parseUniformField(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.parseAttribute(0): not translated, its body couldn't be read: a labelled statement.
        // GLSLParser.getVertAttribute(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.advanceToken(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.advanceToken(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.prevToken(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.prevNode(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.appendToken(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.appendNode(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.getNodeStream(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.getOutput(1): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.createUBOs(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
        // GLSLParser.getSamplerList(0): not translated, its class isn't translated (it uses `LinkedList` (java.util)).
    }

    /// `GLSLParser.Node` (net/vulkanmod/vulkan/shader/converter/GLSLParser.java)
    pub mod glsl_parser_node {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // GLSLParser.Node.getStringValue(0): not translated, it has no body.
    }

    /// `GLSLParser.PreprocessorState` (net/vulkanmod/vulkan/shader/converter/GLSLParser.java)
    pub mod glsl_parser_preprocessor_state {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum PreprocessorState {
            Ignore,
            Default,
        }
        impl PreprocessorState {
            pub const VALUES: [PreprocessorState; 2] = [PreprocessorState::Ignore, PreprocessorState::Default, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    PreprocessorState::Ignore => "IGNORE",
                    PreprocessorState::Default => "DEFAULT",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "IGNORE" => PreprocessorState::Ignore,
                    "DEFAULT" => PreprocessorState::Default,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `GLSLParser.Stage` (net/vulkanmod/vulkan/shader/converter/GLSLParser.java)
    pub mod glsl_parser_stage {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum Stage {
            Vertex,
            Fragment,
        }
        impl Stage {
            pub const VALUES: [Stage; 2] = [Stage::Vertex, Stage::Fragment, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    Stage::Vertex => "VERTEX",
                    Stage::Fragment => "FRAGMENT",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "VERTEX" => Stage::Vertex,
                    "FRAGMENT" => Stage::Fragment,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `GLSLParser.State` (net/vulkanmod/vulkan/shader/converter/GLSLParser.java)
    pub mod glsl_parser_state {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum State {
            Layout,
            Uniform,
            UniformBlock,
            Attribute,
            Default,
        }
        impl State {
            pub const VALUES: [State; 5] = [State::Layout, State::Uniform, State::UniformBlock, State::Attribute, State::Default, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    State::Layout => "LAYOUT",
                    State::Uniform => "UNIFORM",
                    State::UniformBlock => "UNIFORM_BLOCK",
                    State::Attribute => "ATTRIBUTE",
                    State::Default => "DEFAULT",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "LAYOUT" => State::Layout,
                    "UNIFORM" => State::Uniform,
                    "UNIFORM_BLOCK" => State::UniformBlock,
                    "ATTRIBUTE" => State::Attribute,
                    "DEFAULT" => State::Default,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `GLSLParser.TokenNode` (net/vulkanmod/vulkan/shader/converter/GLSLParser.java)
    pub mod glsl_parser_token_node {
        use super::*;
        // The class itself isn't translated: it implements `Node`. Its static parts are below.
        // GLSLParser.TokenNode.TokenNode(2): not translated, its class isn't translated (it implements `Node`).
        // GLSLParser.TokenNode.fromToken(1): not translated, it uses TokenNode, which isn't translated (it implements `Node`).
        // GLSLParser.TokenNode.getStringValue(0): not translated, its class isn't translated (it implements `Node`).
        // GLSLParser.TokenNode.toString(0): not translated, its class isn't translated (it implements `Node`).
    }

    /// `Lexer` (net/vulkanmod/vulkan/shader/converter/Lexer.java)
    pub mod lexer {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Lexer {
            pub input: String,
            pub current_position: i32,
            pub current_char: char,
            pub state: Option<lexer_state::State>,
        }
        impl Lexer {
            pub fn new(input: &str) -> Self {
                let mut s = Self { input: String::new(), current_position: 0i32, current_char: '\0', state: None };
                s.input = input.to_string();
                s.current_position = 0;
                s.current_char = char::from_u32((if !input.is_empty() { (j_char_at(&input, 0) as i32) } else { ('\u{0000}' as i32) }) as u32).unwrap_or('\0');
                s
            }
            pub fn advance_0(&mut self) {
                self.advance_1(1);
            }
            pub fn advance_1(&mut self, i: i32) {
                for j in 0..i {
                    self.current_position += 1;
                    if self.current_position >= j_len(&self.input) {
                        self.current_char = char::from_u32(0 as u32).unwrap_or('\0');
                        break;
                    }
                    self.current_char = j_char_at(&self.input, self.current_position);
                }
            }
            pub fn peek(&self) -> char {
                let peek_position: i32 = self.current_position + 1;
                return char::from_u32((if peek_position < j_len(&self.input) { (j_char_at(&self.input, peek_position) as i32) } else { ('\u{0000}' as i32) }) as u32).unwrap_or('\0');
            }
            // Lexer.tokenize(0): not translated, it stores a second reference to an object (Rust would need it copied or shared).
            // Lexer.nextToken(0): not translated, its body couldn't be read: a labelled statement.
            pub fn line_comment(&mut self) -> token::Token {
                let mut sb: String = String::new();
                sb.push_str("//");
                self.advance_1(2);
                while self.check_eof() && (self.current_char != '\n') {
                    sb.push_str(&(format!("{}", self.current_char)));
                    self.advance_0();
                }
                sb.push_str(&(format!("{}", self.current_char)));
                self.advance_0();
                let value: String = sb.to_string();
                return token::Token::new(token_token_type::TokenType::Comment, Some(value.to_string()));
            }
            pub fn multi_line_comment(&mut self) -> token::Token {
                let mut sb: String = String::new();
                sb.push_str("/*");
                self.advance_1(2);
                while (self.check_eof() && (self.current_char != '*')) && (self.peek() != '/') {
                    sb.push_str(&(format!("{}", self.current_char)));
                    self.advance_0();
                }
                sb.push_str(&(format!("{}", self.current_char)));
                self.advance_0();
                sb.push_str(&(format!("{}", self.current_char)));
                self.advance_0();
                let value: String = sb.to_string();
                return token::Token::new(token_token_type::TokenType::Comment, Some(value.to_string()));
            }
            // Lexer.identifier(0): not translated, it calls `Character`.isJavaIdentifierPart.
            pub fn literal(&mut self) -> token::Token {
                let mut sb: String = String::new();
                while self.current_char.is_ascii_digit() {
                    sb.push_str(&(format!("{}", self.current_char)));
                    self.advance_0();
                }
                if self.current_char == '.' {
                    sb.push_str(&(format!("{}", self.current_char)));
                    self.advance_0();
                }
                while self.current_char.is_ascii_digit() {
                    sb.push_str(&(format!("{}", self.current_char)));
                    self.advance_0();
                }
                let value: String = sb.to_string();
                return token::Token::new(token_token_type::TokenType::Literal, Some(value.to_string()));
            }
            pub fn string(&mut self) -> token::Token {
                let mut sb: String = String::new();
                sb.push_str(&(format!("{}", self.current_char)));
                self.advance_0();
                while self.check_eof() && (self.current_char != '"') {
                    sb.push_str(&(format!("{}", self.current_char)));
                    self.advance_0();
                }
                sb.push_str(&(format!("{}", self.current_char)));
                self.advance_0();
                let value: String = sb.to_string();
                return token::Token::new(token_token_type::TokenType::String, Some(value.to_string()));
            }
            pub fn spacing(&mut self) -> token::Token {
                let mut sb: String = String::new();
                while ((self.current_char as i32) != 0) && self.current_char.is_whitespace() {
                    sb.push_str(&(format!("{}", self.current_char)));
                    self.advance_0();
                }
                let value: String = sb.to_string();
                return token::Token::new(token_token_type::TokenType::Spacing, Some(value.to_string()));
            }
            pub fn check_eof(&self) -> bool {
                return (self.current_char as i32) != 0;
            }
        }
    }

    /// `Lexer.State` (net/vulkanmod/vulkan/shader/converter/Lexer.java)
    pub mod lexer_state {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum State {
            UniformBlock,
            Code,
            Default,
        }
        impl State {
            pub const VALUES: [State; 3] = [State::UniformBlock, State::Code, State::Default, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    State::UniformBlock => "UNIFORM_BLOCK",
                    State::Code => "CODE",
                    State::Default => "DEFAULT",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "UNIFORM_BLOCK" => State::UniformBlock,
                    "CODE" => State::Code,
                    "DEFAULT" => State::Default,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `Sampler` (net/vulkanmod/vulkan/shader/converter/Sampler.java)
    pub mod sampler {
        use super::*;
        // The class itself isn't translated: it implements `Node`. Its static parts are below.
        // Sampler.Sampler(2): not translated, its class isn't translated (it implements `Node`).
        // Sampler.setBinding(1): not translated, its class isn't translated (it implements `Node`).
        // Sampler.getStringValue(0): not translated, its class isn't translated (it implements `Node`).
    }

    /// `Sampler.Type` (net/vulkanmod/vulkan/shader/converter/Sampler.java)
    pub mod sampler_type {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // Sampler.Type.Type(1): not translated, its class isn't translated (its constants carry values).
    }

    /// `Token` (net/vulkanmod/vulkan/shader/converter/Token.java)
    pub mod token {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Token {
            pub r#type: token_token_type::TokenType,
            pub value: Option<String>,
        }
        impl Token {
            pub fn new(r#type: token_token_type::TokenType, value: Option<String>) -> Self {
                let mut s = Self { r#type: r#type, value: value.clone() };
                s.r#type = r#type;
                s.value = value.clone();
                s
            }
            pub fn to_string(&self) -> String {
                return format!("Token{{type={}, value='{}'}}", (self.r#type).java_name(), self.value.as_ref().map_or_else(|| "null".to_string(), |v| v.to_string()));
            }
        }
    }

    /// `Token.TokenType` (net/vulkanmod/vulkan/shader/converter/Token.java)
    pub mod token_token_type {
        use super::*;
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        pub enum TokenType {
            Preprocessor,
            Keyword,
            Identifier,
            Literal,
            Operator,
            Punctuation,
            String,
            Spacing,
            Comment,
            LeftBrace,
            RightBrace,
            LeftParenthesis,
            RightParenthesis,
            Colon,
            Semicolon,
            Dot,
            Comma,
            Type,
            Layout,
            Eof,
        }
        impl TokenType {
            pub const VALUES: [TokenType; 20] = [TokenType::Preprocessor, TokenType::Keyword, TokenType::Identifier, TokenType::Literal, TokenType::Operator, TokenType::Punctuation, TokenType::String, TokenType::Spacing, TokenType::Comment, TokenType::LeftBrace, TokenType::RightBrace, TokenType::LeftParenthesis, TokenType::RightParenthesis, TokenType::Colon, TokenType::Semicolon, TokenType::Dot, TokenType::Comma, TokenType::Type, TokenType::Layout, TokenType::Eof, ];
            /// Java's `name()`.
            pub fn java_name(self) -> &'static str {
                match self {
                    TokenType::Preprocessor => "PREPROCESSOR",
                    TokenType::Keyword => "KEYWORD",
                    TokenType::Identifier => "IDENTIFIER",
                    TokenType::Literal => "LITERAL",
                    TokenType::Operator => "OPERATOR",
                    TokenType::Punctuation => "PUNCTUATION",
                    TokenType::String => "STRING",
                    TokenType::Spacing => "SPACING",
                    TokenType::Comment => "COMMENT",
                    TokenType::LeftBrace => "LEFT_BRACE",
                    TokenType::RightBrace => "RIGHT_BRACE",
                    TokenType::LeftParenthesis => "LEFT_PARENTHESIS",
                    TokenType::RightParenthesis => "RIGHT_PARENTHESIS",
                    TokenType::Colon => "COLON",
                    TokenType::Semicolon => "SEMICOLON",
                    TokenType::Dot => "DOT",
                    TokenType::Comma => "COMMA",
                    TokenType::Type => "TYPE",
                    TokenType::Layout => "LAYOUT",
                    TokenType::Eof => "EOF",
                }
            }
            /// Java's `valueOf`: panics on a name that isn't one, as Java throws.
            pub fn value_of(name: &str) -> Self {
                match name {
                    "PREPROCESSOR" => TokenType::Preprocessor,
                    "KEYWORD" => TokenType::Keyword,
                    "IDENTIFIER" => TokenType::Identifier,
                    "LITERAL" => TokenType::Literal,
                    "OPERATOR" => TokenType::Operator,
                    "PUNCTUATION" => TokenType::Punctuation,
                    "STRING" => TokenType::String,
                    "SPACING" => TokenType::Spacing,
                    "COMMENT" => TokenType::Comment,
                    "LEFT_BRACE" => TokenType::LeftBrace,
                    "RIGHT_BRACE" => TokenType::RightBrace,
                    "LEFT_PARENTHESIS" => TokenType::LeftParenthesis,
                    "RIGHT_PARENTHESIS" => TokenType::RightParenthesis,
                    "COLON" => TokenType::Colon,
                    "SEMICOLON" => TokenType::Semicolon,
                    "DOT" => TokenType::Dot,
                    "COMMA" => TokenType::Comma,
                    "TYPE" => TokenType::Type,
                    "LAYOUT" => TokenType::Layout,
                    "EOF" => TokenType::Eof,
                    _ => panic!("No enum constant {name}"),
                }
            }
        }
    }

    /// `UniformBlock` (net/vulkanmod/vulkan/shader/converter/UniformBlock.java)
    pub mod uniform_block {
        use super::*;
        // The class itself isn't translated: it implements `Node`. Its static parts are below.
        // UniformBlock.UniformBlock(1): not translated, its class isn't translated (it implements `Node`).
        // UniformBlock.addField(1): not translated, its class isn't translated (it implements `Node`).
        // UniformBlock.setBinding(1): not translated, its class isn't translated (it implements `Node`).
        // UniformBlock.setAlias(1): not translated, its class isn't translated (it implements `Node`).
        // UniformBlock.getStringValue(0): not translated, its class isn't translated (it implements `Node`).
    }

    /// `UniformBlock.Field` (net/vulkanmod/vulkan/shader/converter/UniformBlock.java)
    pub mod uniform_block_field {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Field {
            pub r#type: String,
            pub name: String,
        }
        impl Field {
            pub fn new(r#type: &str, name: &str) -> Self {
                let mut s = Self { r#type: String::new(), name: String::new() };
                s.r#type = (r#type).to_string();
                s.name = name.to_string();
                s
            }
        }
    }

    /// `DescriptorSets` (net/vulkanmod/vulkan/shader/DescriptorSets.java)
    pub mod descriptor_sets {
        use super::*;
        // The class itself isn't translated: it uses `IntBuffer` (java.nio). Its static parts are below.
        // DEVICE: not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        // DescriptorSets.DescriptorSets(1): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.bindSets(3): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.updateUniforms(1): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.needsUpdate(1): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.checkPoolSize(1): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.updateDescriptorSet(2): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.createDescriptorSets(1): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.createDescriptorPool(1): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.resetIdx(0): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
        // DescriptorSets.cleanUp(0): not translated, its class isn't translated (it uses `IntBuffer` (java.nio)).
    }

    /// `Descriptor` (net/vulkanmod/vulkan/shader/descriptor/Descriptor.java)
    pub mod descriptor {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // Descriptor.getBinding(0): not translated, it has no body.
        // Descriptor.getType(0): not translated, it has no body.
        // Descriptor.getStages(0): not translated, it has no body.
    }

    /// `ImageDescriptor` (net/vulkanmod/vulkan/shader/descriptor/ImageDescriptor.java)
    pub mod image_descriptor {
        use super::*;
        // The class itself isn't translated: it implements `Descriptor`. Its static parts are below.
        // ImageDescriptor.ImageDescriptor(5): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.getBinding(0): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.getType(0): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.getStages(0): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.setLayout(1): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.getLayout(0): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.setMipLevel(1): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.getMipLevel(0): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.getImage(0): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.getImageView(1): not translated, its class isn't translated (it implements `Descriptor`).
        // ImageDescriptor.isStorageImage(0): not translated, its class isn't translated (it implements `Descriptor`).
    }

    /// `ImageDescriptor.State` (net/vulkanmod/vulkan/shader/descriptor/ImageDescriptor.java)
    pub mod image_descriptor_state {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct State {
            pub image_view: i64,
            pub sampler: i64,
        }
        impl State {
            pub fn new(image_view: i64, sampler: i64) -> Self {
                let mut s = Self { image_view: 0i64, sampler: 0i64 };
                s.set(image_view, sampler);
                s
            }
            pub fn set(&mut self, image_view: i64, sampler: i64) {
                self.image_view = image_view;
                self.sampler = sampler;
            }
            pub fn is_current_state(&self, image_view: i64, sampler: i64) -> bool {
                return (self.image_view == image_view) && (self.sampler == sampler);
            }
        }
    }

    /// `ManualUBO` (net/vulkanmod/vulkan/shader/descriptor/ManualUBO.java)
    pub mod manual_ubo {
        use super::*;
        // The class itself isn't translated: it extends `UBO`. Its static parts are below.
        // ManualUBO.ManualUBO(3): not translated, its class isn't translated (it extends `UBO`).
        // ManualUBO.update(1): not translated, its class isn't translated (it extends `UBO`).
        // ManualUBO.setSrc(2): not translated, its class isn't translated (it extends `UBO`).
        // ManualUBO.setUpdate(1): not translated, its class isn't translated (it extends `UBO`).
    }

    /// `UBO` (net/vulkanmod/vulkan/shader/descriptor/UBO.java)
    pub mod ubo {
        use super::*;
        // The class itself isn't translated: it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout). Its static parts are below.
        // UBO.UBO(5): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.toString(0): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.getBinding(0): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.getType(0): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.getStages(0): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.getBufferSlice(0): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.useGlobalBuffer(0): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.setUseGlobalBuffer(1): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.shouldUpdate(0): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // UBO.setUpdate(1): not translated, its class isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
    }

    /// `GraphicsPipeline` (net/vulkanmod/vulkan/shader/GraphicsPipeline.java)
    pub mod graphics_pipeline {
        use super::*;
        // The class itself isn't translated: it extends `Pipeline`. Its static parts are below.
        // GraphicsPipeline.GraphicsPipeline(1): not translated, its class isn't translated (it extends `Pipeline`).
        // GraphicsPipeline.getHandle(1): not translated, its class isn't translated (it extends `Pipeline`).
        // GraphicsPipeline.createGraphicsPipeline(1): not translated, its class isn't translated (it extends `Pipeline`).
        // GraphicsPipeline.createShaderModules(1): not translated, its class isn't translated (it extends `Pipeline`).
        // GraphicsPipeline.cleanUp(0): not translated, its class isn't translated (it extends `Pipeline`).
        // GraphicsPipeline.getBindingDescription(1): not translated, it uses `Buffer` (org.lwjgl.vulkan.VkPipelineShaderStageCreateInfo).
        // GraphicsPipeline.getAttributeDescriptions(1): not translated, it uses `Buffer` (org.lwjgl.vulkan.VkPipelineShaderStageCreateInfo).
    }

    /// `GraphicsPipeline.VertexInputDescription` (net/vulkanmod/vulkan/shader/GraphicsPipeline.java)
    pub mod graphics_pipeline_vertex_input_description {
        use super::*;
        // The class itself isn't translated: it uses `Buffer` (org.lwjgl.vulkan.VkPipelineShaderStageCreateInfo). Its static parts are below.
        // GraphicsPipeline.VertexInputDescription.VertexInputDescription(1): not translated, its class isn't translated (it uses `Buffer` (org.lwjgl.vulkan.VkPipelineShaderStageCreateInfo)).
        // GraphicsPipeline.VertexInputDescription.cleanUp(0): not translated, its class isn't translated (it uses `Buffer` (org.lwjgl.vulkan.VkPipelineShaderStageCreateInfo)).
    }

    /// `AlignedStruct` (net/vulkanmod/vulkan/shader/layout/AlignedStruct.java)
    pub mod aligned_struct {
        use super::*;
        // The class itself isn't translated: it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be)). Its static parts are below.
        // AlignedStruct.AlignedStruct(2): not translated, it stores a second reference to an object (Rust would need it copied or shared).
        // AlignedStruct.update(1): not translated, its class isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
        // AlignedStruct.getUniforms(0): not translated, its class isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
        // AlignedStruct.getSize(0): not translated, its class isn't translated (it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be))).
        // AlignedStruct.builder(0): not translated, it uses Builder, which isn't translated (it uses Info, which isn't translated (a function kept in a variable or field (only parameters can be))).
    }

    /// `AlignedStruct.Builder` (net/vulkanmod/vulkan/shader/layout/AlignedStruct.java)
    pub mod aligned_struct_builder {
        use super::*;
        // The class itself isn't translated: it uses Info, which isn't translated (a function kept in a variable or field (only parameters can be)). Its static parts are below.
        // AlignedStruct.Builder.addUniform(3): not translated, `this` used as a value.
        // AlignedStruct.Builder.addUniform(2): not translated, `this` used as a value.
        // AlignedStruct.Builder.addUniform(1): not translated, it stores a second reference to an object (Rust would need it copied or shared).
        // AlignedStruct.Builder.buildUBO(2): not translated, it uses UBO, which isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // AlignedStruct.Builder.buildUBO(3): not translated, it uses UBO, which isn't translated (it extends `AlignedStruct` (net.vulkanmod.vulkan.shader.layout)).
        // AlignedStruct.Builder.buildPushConstant(1): not translated, it uses PushConstants, which isn't translated (it extends `AlignedStruct`).
    }

    /// `Mat3` (net/vulkanmod/vulkan/shader/layout/Mat3.java)
    pub mod mat3 {
        use super::*;
        // The class itself isn't translated: it extends `Uniform`. Its static parts are below.
        // Mat3.Mat3(1): not translated, its class isn't translated (it extends `Uniform`).
        // Mat3.update(1): not translated, its class isn't translated (it extends `Uniform`).
    }

    /// `PushConstants` (net/vulkanmod/vulkan/shader/layout/PushConstants.java)
    pub mod push_constants {
        use super::*;
        // The class itself isn't translated: it extends `AlignedStruct`. Its static parts are below.
        // PushConstants.PushConstants(3): not translated, its class isn't translated (it extends `AlignedStruct`).
    }

    /// `Uniform` (net/vulkanmod/vulkan/shader/layout/Uniform.java)
    pub mod uniform {
        use super::*;
        // The class itself isn't translated: a function kept in a variable or field (only parameters can be). Its static parts are below.
        // Uniform.Uniform(1): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.setupSupplier(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.setSupplier(1): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.getName(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.update(1): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.createField(1): not translated, it uses Uniform, which isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.getOffset(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.getSize(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.getInfo(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.toString(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.createUniformInfo(3): not translated, a switch expression giving a Uniform.Info.
        // Uniform.createUniformInfo(2): not translated, a switch expression giving a Uniform.Info.
    }

    /// `Uniform.Info` (net/vulkanmod/vulkan/shader/layout/Uniform.java)
    pub mod uniform_info {
        use super::*;
        // The class itself isn't translated: a function kept in a variable or field (only parameters can be). Its static parts are below.
        // Uniform.Info.Info(4): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.Info.getSizeBytes(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.Info.computeAlignmentOffset(1): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.Info.setupSupplier(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.Info.hasSupplier(0): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
        // Uniform.Info.setBufferSupplier(1): not translated, its class isn't translated (a function kept in a variable or field (only parameters can be)).
    }

    /// `Vec1f` (net/vulkanmod/vulkan/shader/layout/Vec1f.java)
    pub mod vec1f {
        use super::*;
        // The class itself isn't translated: it extends `Uniform`. Its static parts are below.
        // Vec1f.Vec1f(1): not translated, its class isn't translated (it extends `Uniform`).
        // Vec1f.setupSupplier(0): not translated, its class isn't translated (it extends `Uniform`).
        // Vec1f.setSupplier(1): not translated, its class isn't translated (it extends `Uniform`).
        // Vec1f.update(1): not translated, its class isn't translated (it extends `Uniform`).
    }

    /// `Vec1i` (net/vulkanmod/vulkan/shader/layout/Vec1i.java)
    pub mod vec1i {
        use super::*;
        // The class itself isn't translated: it extends `Uniform`. Its static parts are below.
        // Vec1i.Vec1i(1): not translated, its class isn't translated (it extends `Uniform`).
        // Vec1i.setupSupplier(0): not translated, its class isn't translated (it extends `Uniform`).
        // Vec1i.setSupplier(1): not translated, its class isn't translated (it extends `Uniform`).
        // Vec1i.update(1): not translated, its class isn't translated (it extends `Uniform`).
    }

    /// `Pipeline` (net/vulkanmod/vulkan/shader/Pipeline.java)
    pub mod pipeline {
        use super::*;
        // The class itself isn't translated: it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio)). Its static parts are below.
        // DEVICE: not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        // PIPELINE_CACHE: not translated, it calls Pipeline.createPipelineCache, which isn't translated.
        // PIPELINES: not translated, it uses Pipeline, which isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.createPipelineCache(0): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Pipeline.destroyPipelineCache(0): not translated, it uses Pipeline.DEVICE, which isn't translated (it uses `VkDevice` (org.lwjgl.vulkan)).
        // Pipeline.recreateDescriptorSets(1): not translated, it uses Pipeline, which isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.Pipeline(1): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.createDescriptorSetLayout(0): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.createPipelineLayout(0): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.createDescriptorSets(1): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.scheduleCleanUp(0): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.cleanUp(0): not translated, it has no body.
        // Pipeline.destroyDescriptorSets(0): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.resetDescriptorPool(1): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getPushConstants(0): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getLayout(0): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getBuffers(0): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getUBO(1): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getUBO(1): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getUBO(1): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getImageDescriptor(1): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getImageDescriptor(1): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.getImageDescriptors(0): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.bindDescriptorSets(2): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.bindDescriptorSets(3): not translated, its class isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // Pipeline.createShaderModule(1): not translated, it uses `ByteBuffer` (java.nio).
        // Pipeline.builder(2): not translated, it uses Minecraft's `VertexFormat`.
    }

    /// `Pipeline.Builder` (net/vulkanmod/vulkan/shader/Pipeline.java)
    pub mod pipeline_builder {
        use super::*;
        // The class itself isn't translated: it uses Minecraft's `VertexFormat`. Its static parts are below.
        // Pipeline.Builder.Builder(2): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.Builder(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.Builder(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.Builder(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.setUniforms(2): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.setShaderSrc(2): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.addUBO(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.addImageDescriptor(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.applyConfig(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.setUniformSupplierGetter(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.parseUboNode(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.parseImageDescriptor(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.parsePushConstantNode(1): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.getUBOs(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.getImageDescriptors(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.getPushConstants(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.getShadersSrc(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
        // Pipeline.Builder.createGraphicsPipeline(0): not translated, its class isn't translated (it uses Minecraft's `VertexFormat`).
    }

    /// `PipelineConfig` (net/vulkanmod/vulkan/shader/PipelineConfig.java)
    pub mod pipeline_config {
        use super::*;
        // The class itself isn't translated: it uses `EnumMap` (java.util). Its static parts are below.
        // PipelineConfig.PipelineConfig(4): not translated, its class isn't translated (it uses `EnumMap` (java.util)).
        // PipelineConfig.fromJson(2): not translated, it uses PipelineConfig, which isn't translated (it uses `EnumMap` (java.util)).
        // PipelineConfig.parseUboNode(1): not translated, it calls Minecraft's `GsonHelper`.convertToJsonObject.
        // PipelineConfig.parseManualUboNode(1): not translated, it calls Minecraft's `GsonHelper`.convertToJsonObject.
        // PipelineConfig.parseSamplerNode(2): not translated, it calls Minecraft's `GsonHelper`.convertToJsonObject.
        // PipelineConfig.parsePushConstantNode(1): not translated, it calls Minecraft's `GsonHelper`.convertToJsonObject.
        // PipelineConfig.builder(0): not translated, it uses Builder, which isn't translated (it uses `EnumMap` (java.util)).
        pub fn get_stage_from_string(s: &str) -> i32 {
            return match &*s {
                "vertex" => 1,
                "fragment" => 16,
                "all" => 31,
                "compute" => 32,
                _ => {
                panic!("{}", "cannot identify type..");
                },
            };
        }
        // PipelineConfig.convertType(2): not translated, a switch case that falls through.
    }

    /// `PipelineConfig.Builder` (net/vulkanmod/vulkan/shader/PipelineConfig.java)
    pub mod pipeline_config_builder {
        use super::*;
        // The class itself isn't translated: it uses `EnumMap` (java.util). Its static parts are below.
        // PipelineConfig.Builder.withShader(2): not translated, its class isn't translated (it uses `EnumMap` (java.util)).
        // PipelineConfig.Builder.addUB(1): not translated, its class isn't translated (it uses `EnumMap` (java.util)).
        // PipelineConfig.Builder.addImageDescriptor(4): not translated, its class isn't translated (it uses `EnumMap` (java.util)).
        // PipelineConfig.Builder.addImageDescriptor(1): not translated, its class isn't translated (it uses `EnumMap` (java.util)).
        // PipelineConfig.Builder.setPushConstants(1): not translated, its class isn't translated (it uses `EnumMap` (java.util)).
        // PipelineConfig.Builder.build(0): not translated, its class isn't translated (it uses `EnumMap` (java.util)).
    }

    /// `PipelineConfig.DescriptorBinding` (net/vulkanmod/vulkan/shader/PipelineConfig.java)
    pub mod pipeline_config_descriptor_binding {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // PipelineConfig.DescriptorBinding.getBinding(0): not translated, it has no body.
    }

    /// `PipelineConfig.ImageDescriptorInfo` (net/vulkanmod/vulkan/shader/PipelineConfig.java)
    pub mod pipeline_config_image_descriptor_info {
        use super::*;
        #[derive(Clone, Debug, PartialEq)]
        pub struct ImageDescriptorInfo {
            pub binding: i32,
            pub r#type: String,
            pub name: String,
            pub image_idx: i32,
        }
        impl ImageDescriptorInfo {
            pub fn new(binding: i32, r#type: &str, name: &str, image_idx: i32) -> Self {
                let mut s = Self { binding: binding, r#type: String::new(), name: String::new(), image_idx: image_idx };
                s.binding = binding;
                s.r#type = (r#type).to_string();
                s.name = name.to_string();
                s.image_idx = image_idx;
                s
            }
            pub fn binding(&self) -> i32 {
                return self.binding;
            }
            pub fn r#type(&self) -> String {
                return (self.r#type).to_string();
            }
            pub fn name(&self) -> String {
                return self.name.to_string();
            }
            pub fn image_idx(&self) -> i32 {
                return self.image_idx;
            }
        }
    }

    /// `PipelineConfig.UB` (net/vulkanmod/vulkan/shader/PipelineConfig.java)
    pub mod pipeline_config_ub {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct UB {
            pub binding: i32,
            pub stage: i32,
            pub size: i32,
            pub uniforms: Vec<pipeline_config_uniform::Uniform>,
        }
        impl UB {
            pub fn new(binding: i32, stage: i32, size: i32, uniforms: &[pipeline_config_uniform::Uniform]) -> Self {
                let mut s = Self { binding: binding, stage: stage, size: size, uniforms: Vec::new() };
                s.binding = binding;
                s.stage = stage;
                s.size = size;
                s.uniforms = uniforms.to_vec();
                s
            }
        }
        pub fn builder(binding: i32, stage: i32) -> pipeline_config_ub_builder::Builder {
            return pipeline_config_ub_builder::Builder::new(binding, stage);
        }
    }

    /// `PipelineConfig.UB.Builder` (net/vulkanmod/vulkan/shader/PipelineConfig.java)
    pub mod pipeline_config_ub_builder {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Builder {
            pub binding: i32,
            pub stage: i32,
            pub size: i32,
            pub uniforms: Vec<pipeline_config_uniform::Uniform>,
        }
        impl Builder {
            pub fn new(binding: i32, stage: i32) -> Self {
                let mut s = Self { binding: binding, stage: stage, size: 0i32, uniforms: Vec::new() };
                s.binding = binding;
                s.stage = stage;
                s
            }
            // PipelineConfig.UB.Builder.addUniform(2): not translated, `this` used as a value.
            // PipelineConfig.UB.Builder.setSize(1): not translated, `this` used as a value.
            pub fn build(&self) -> pipeline_config_ub::UB {
                return pipeline_config_ub::UB::new(self.binding, self.stage, self.size, &self.uniforms);
            }
        }
    }

    /// `PipelineConfig.Uniform` (net/vulkanmod/vulkan/shader/PipelineConfig.java)
    pub mod pipeline_config_uniform {
        use super::*;
        #[derive(Clone, Debug, PartialEq)]
        pub struct Uniform {
            pub r#type: String,
            pub name: String,
        }
        impl Uniform {
            pub fn new(r#type: &str, name: &str) -> Self {
                let mut s = Self { r#type: String::new(), name: String::new() };
                s.r#type = (r#type).to_string();
                s.name = name.to_string();
                s
            }
            pub fn r#type(&self) -> String {
                return (self.r#type).to_string();
            }
            pub fn name(&self) -> String {
                return self.name.to_string();
            }
        }
    }

    /// `PipelineState` (net/vulkanmod/vulkan/shader/PipelineState.java)
    pub mod pipeline_state {
        use super::*;
        // The class itself isn't translated: it uses RenderPass, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects))). Its static parts are below.
        pub const DEFAULT_DEPTH_OP: i32 = 515;
        pub static BLEND_INFO: LazyLock<JCell<pipeline_state_blend_info::BlendInfo>> = LazyLock::new(|| JCell::new(pipeline_state::default_blend_info()));
        // DEFAULT: not translated, it uses PipelineState, which isn't translated (it uses RenderPass, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)))).
        // currentState: not translated, it uses PipelineState, which isn't translated (it uses RenderPass, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)))).
        // PipelineState.getCurrentPipelineState(1): not translated, it uses PipelineState, which isn't translated (it uses RenderPass, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)))).
        // PipelineState.getBlendState(0): not translated, a changing static passed to a method.
        pub fn get_assembly_raster_state() -> i32 {
            return pipeline_state_assembly_raster_state::encode(v_render_system::CULL.get(), v_render_system::TOPOLOGY.get(), v_render_system::POLYGON_MODE.get());
        }
        // PipelineState.getDepthState(0): not translated, it calls DepthState.encodeDepthFun, which isn't translated.
        pub fn get_logic_op_state() -> i32 {
            let mut logic_op_state: i32 = 0;
            logic_op_state |= if v_render_system::LOGIC_OP.get() { 1 } else { 0 };
            return logic_op_state | (pipeline_state_logic_op_state::encode_logic_op_fun(v_render_system::LOGIC_OP_FUN.get()));
        }
        // PipelineState.PipelineState(6): not translated, its class isn't translated (it uses RenderPass, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)))).
        // PipelineState.checkEquals(6): not translated, its class isn't translated (it uses RenderPass, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)))).
        // PipelineState.equals(1): not translated, its class isn't translated (it uses RenderPass, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)))).
        // PipelineState.hashCode(0): not translated, its class isn't translated (it uses RenderPass, which isn't translated (it uses Framebuffer, which isn't translated (it uses `Reference2LongArrayMap` (it.unimi.dsi.fastutil.objects)))).
        pub fn default_blend_info() -> pipeline_state_blend_info::BlendInfo {
            return pipeline_state_blend_info::BlendInfo::new(true, 6, 7, 1, 0, 0);
        }
    }

    /// `PipelineState.AssemblyRasterState` (net/vulkanmod/vulkan/shader/PipelineState.java)
    pub mod pipeline_state_assembly_raster_state {
        use super::*;
        pub const POLYGON_MODE_MASK: i32 = 7;
        pub const TOPOLOGY_OFFSET: i32 = 3;
        pub const TOPOLOGY_BITS: i32 = 4;
        pub const TOPOLOGY_MASK: i32 = 31;
        pub const CULL_MODE_OFFSET: i32 = 7;
        pub const CULL_MODE_BITS: i32 = 2;
        pub const CULL_MODE_MASK: i32 = 3;
        #[derive(Clone, Debug)]
        pub struct AssemblyRasterState {
        }
        impl AssemblyRasterState {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn encode(cull: bool, topology: i32, polygon_mode: i32) -> i32 {
            let state: i32 = polygon_mode | (topology << 3);
            return state | ((if cull { 2 } else { 0 }) << 7);
        }
        pub fn decode_topology(state: i32) -> i32 {
            return (((state as u32) >> 3) as i32) & 31;
        }
        pub fn decode_polygon_mode(state: i32) -> i32 {
            return state & 7;
        }
        pub fn decode_cull_mode(state: i32) -> i32 {
            return (((state as u32) >> 7) as i32) & 3;
        }
    }

    /// `PipelineState.BlendInfo` (net/vulkanmod/vulkan/shader/PipelineState.java)
    pub mod pipeline_state_blend_info {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct BlendInfo {
            pub enabled: bool,
            pub src_rgb_factor: i32,
            pub dst_rgb_factor: i32,
            pub src_alpha_factor: i32,
            pub dst_alpha_factor: i32,
            pub blend_op: i32,
        }
        impl BlendInfo {
            pub fn new(enabled: bool, src_rgb_factor: i32, dst_rgb_factor: i32, src_alpha_factor: i32, dst_alpha_factor: i32, blend_op: i32) -> Self {
                let mut s = Self { enabled: enabled, src_rgb_factor: src_rgb_factor, dst_rgb_factor: dst_rgb_factor, src_alpha_factor: src_alpha_factor, dst_alpha_factor: dst_alpha_factor, blend_op: blend_op };
                s.enabled = enabled;
                s.src_rgb_factor = src_rgb_factor;
                s.dst_rgb_factor = dst_rgb_factor;
                s.src_alpha_factor = src_alpha_factor;
                s.dst_alpha_factor = dst_alpha_factor;
                s.blend_op = blend_op;
                s
            }
            pub fn set_blend_function(&mut self, source_factor: i32, dest_factor: i32) {
                self.src_rgb_factor = pipeline_state_blend_info::gl_to_vulkan_blend_factor(source_factor);
                self.src_alpha_factor = pipeline_state_blend_info::gl_to_vulkan_blend_factor(source_factor);
                self.dst_rgb_factor = pipeline_state_blend_info::gl_to_vulkan_blend_factor(dest_factor);
                self.dst_alpha_factor = pipeline_state_blend_info::gl_to_vulkan_blend_factor(dest_factor);
            }
            pub fn set_blend_func_separate(&mut self, src_rgb: i32, dst_rgb: i32, src_alpha: i32, dst_alpha: i32) {
                self.src_rgb_factor = pipeline_state_blend_info::gl_to_vulkan_blend_factor(src_rgb);
                self.src_alpha_factor = pipeline_state_blend_info::gl_to_vulkan_blend_factor(src_alpha);
                self.dst_rgb_factor = pipeline_state_blend_info::gl_to_vulkan_blend_factor(dst_rgb);
                self.dst_alpha_factor = pipeline_state_blend_info::gl_to_vulkan_blend_factor(dst_alpha);
            }
            pub fn set_blend_op(&mut self, i: i32) {
                self.blend_op = pipeline_state_blend_info::gl_to_vulkan_blend_op(i);
            }
            // PipelineState.BlendInfo.createBlendState(0): not translated, `this` used as a value.
        }
        pub fn gl_to_vulkan_blend_op(value: i32) -> i32 {
            return match value {
                32774 => 0,
                32775 => 3,
                32776 => 4,
                _ => {
                panic!("{}", format!("unknown blend factor: {}", value));
                },
                32778 => 1,
                32779 => 2,
            };
        }
        pub fn gl_to_vulkan_blend_factor(value: i32) -> i32 {
            return match value {
                0 => 0,
                1 => 1,
                768 => 2,
                769 => 3,
                770 => 6,
                771 => 7,
                772 => 8,
                773 => 9,
                774 => 4,
                775 => 5,
                776 => 14,
                _ => {
                panic!("{}", format!("unknown blend factor: {}", value));
                },
            };
        }
    }

    /// `PipelineState.BlendState` (net/vulkanmod/vulkan/shader/PipelineState.java)
    pub mod pipeline_state_blend_state {
        use super::*;
        pub const SRC_RGB_OFFSET: i32 = 0;
        pub const DST_RGB_OFFSET: i32 = 5;
        pub const SRC_A_OFFSET: i32 = 10;
        pub const DST_A_OFFSET: i32 = 15;
        pub const FUN_OFFSET: i32 = 20;
        pub const ENABLE_BIT: i32 = 16777216;
        pub const OP_MASK: i32 = 15;
        pub const FACTOR_MASK: i32 = 31;
        #[derive(Clone, Debug)]
        pub struct BlendState {
        }
        impl BlendState {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn get_state(blend_info: &pipeline_state_blend_info::BlendInfo) -> i32 {
            let mut s: i32 = 0;
            s |= if blend_info.enabled { 16777216 } else { 0 };
            s |= pipeline_state_blend_state::encode(blend_info.src_rgb_factor, 0, 31);
            s |= pipeline_state_blend_state::encode(blend_info.dst_rgb_factor, 5, 31);
            s |= pipeline_state_blend_state::encode(blend_info.src_alpha_factor, 10, 31);
            s |= pipeline_state_blend_state::encode(blend_info.dst_alpha_factor, 15, 31);
            return s | pipeline_state_blend_state::encode(blend_info.blend_op, 20, 15);
        }
        pub fn enable(i: i32) -> bool {
            return (i & 16777216) != 0;
        }
        pub fn encode(i: i32, offset: i32, mask: i32) -> i32 {
            return (i & mask) << offset;
        }
        pub fn decode(i: i32, offset: i32, bits: i32) -> i32 {
            return (((i as u32) >> offset) as i32) & bits;
        }
        pub fn get_src_rgb_factor(s: i32) -> i32 {
            return pipeline_state_blend_state::decode(s, 0, 31);
        }
        pub fn get_dst_rgb_factor(s: i32) -> i32 {
            return pipeline_state_blend_state::decode(s, 5, 31);
        }
        pub fn get_src_alpha_factor(s: i32) -> i32 {
            return pipeline_state_blend_state::decode(s, 10, 31);
        }
        pub fn get_dst_alpha_factor(s: i32) -> i32 {
            return pipeline_state_blend_state::decode(s, 15, 31);
        }
        pub fn blend_op(state: i32) -> i32 {
            return pipeline_state_blend_state::decode(state, 20, 15);
        }
    }

    /// `PipelineState.ColorMask` (net/vulkanmod/vulkan/shader/PipelineState.java)
    pub mod pipeline_state_color_mask {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct ColorMask {
        }
        impl ColorMask {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn get_color_mask(r: bool, g: bool, b: bool, a: bool) -> i32 {
            return (((if r { 1 } else { 0 }) | (if g { 2 } else { 0 })) | (if b { 4 } else { 0 })) | (if a { 8 } else { 0 });
        }
    }

    /// `PipelineState.DepthState` (net/vulkanmod/vulkan/shader/PipelineState.java)
    pub mod pipeline_state_depth_state {
        use super::*;
        pub const DEPTH_TEST_BIT: i32 = 1;
        pub const DEPTH_MASK_BIT: i32 = 2;
        pub const DEPTH_FUN_OFFSET: i32 = 2;
        pub const DEPTH_FUN_BITS: i32 = 4;
        #[derive(Clone, Debug)]
        pub struct DepthState {
        }
        impl DepthState {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn depth_test(i: i32) -> bool {
            return (i & 1) != 0;
        }
        pub fn depth_mask(i: i32) -> bool {
            return (i & 2) != 0;
        }
        // PipelineState.DepthState.encodeDepthFun(1): not translated, it calls DepthState.glToVulkan, which isn't translated.
        pub fn decode_depth_fun(state: i32) -> i32 {
            return ((state as u32) >> 2) as i32;
        }
        // PipelineState.DepthState.glToVulkan(1): not translated, String.formatted with 1 values.
    }

    /// `PipelineState.LogicOpState` (net/vulkanmod/vulkan/shader/PipelineState.java)
    pub mod pipeline_state_logic_op_state {
        use super::*;
        pub const ENABLE_BIT: i32 = 1;
        pub const FUN_OFFSET: i32 = 1;
        pub const FUN_BITS: i32 = 5;
        #[derive(Clone, Debug)]
        pub struct LogicOpState {
        }
        impl LogicOpState {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn enable(i: i32) -> bool {
            return (i & 1) != 0;
        }
        pub fn encode_logic_op_fun(gl_fun: i32) -> i32 {
            let fun: i32 = pipeline_state_logic_op_state::gl_to_vulkan(gl_fun);
            return fun << 1;
        }
        pub fn decode_fun(state: i32) -> i32 {
            return ((state as u32) >> 1) as i32;
        }
        pub fn gl_to_vulkan(f: i32) -> i32 {
            return match f {
                5377 => 1,
                5378 => 2,
                5379 => 3,
                5380 => 4,
                5381 => 5,
                5382 => 6,
                5383 => 7,
                5384 => 8,
                5385 => 9,
                5386 => 10,
                5387 => 11,
                5388 => 12,
                5389 => 13,
                5390 => 14,
                5391 => 15,
                _ => 1,
            };
        }
    }

    /// `SPIRVUtils` (net/vulkanmod/vulkan/shader/SPIRVUtils.java)
    pub mod spirv_utils {
        use super::*;
        pub const DEBUG: bool = true;
        pub const OPTIMIZATIONS: bool = false;
        pub static COMPILER: JCell<i64> = JCell::new(0i64);
        pub static OPTIONS: JCell<i64> = JCell::new(0i64);
        // SHADER_INCLUDER: not translated, it uses ShaderIncluder, which isn't translated (it implements `ShadercIncludeResolveI` (org.lwjgl.util.shaderc)).
        // SHADER_RELEASER: not translated, it uses ShaderReleaser, which isn't translated (it implements `ShadercIncludeResultReleaseI` (org.lwjgl.util.shaderc)).
        pub const P_USER_DATA: i64 = 0i64;
        // includePaths: not translated, it uses `ObjectArrayList` (it.unimi.dsi.fastutil.objects).
        #[derive(Clone, Debug)]
        pub struct SPIRVUtils {
        }
        impl SPIRVUtils {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // SPIRVUtils.initCompiler(0): not translated, it calls `Shaderc` (org.lwjgl.util.shaderc).shaderc_compiler_initialize.
        // SPIRVUtils.addIncludePath(1): not translated, it uses `URL` (java.net).
        // SPIRVUtils.compileShader(3): not translated, it uses SPIRV, which isn't translated (it implements `NativeResource` (org.lwjgl.system)).
    }

    /// `SPIRVUtils.SPIRV` (net/vulkanmod/vulkan/shader/SPIRVUtils.java)
    pub mod spirv_utils_spirv {
        use super::*;
        // The class itself isn't translated: it implements `NativeResource` (org.lwjgl.system). Its static parts are below.
        // SPIRVUtils.SPIRV.SPIRV(2): not translated, its class isn't translated (it implements `NativeResource` (org.lwjgl.system)).
        // SPIRVUtils.SPIRV.bytecode(0): not translated, its class isn't translated (it implements `NativeResource` (org.lwjgl.system)).
        // SPIRVUtils.SPIRV.free(0): not translated, its class isn't translated (it implements `NativeResource` (org.lwjgl.system)).
    }

    /// `SPIRVUtils.ShaderIncluder` (net/vulkanmod/vulkan/shader/SPIRVUtils.java)
    pub mod spirv_utils_shader_includer {
        use super::*;
        // The class itself isn't translated: it implements `ShadercIncludeResolveI` (org.lwjgl.util.shaderc). Its static parts are below.
        pub const MAX_PATH_LENGTH: i32 = 4096;
        // SPIRVUtils.ShaderIncluder.invoke(5): not translated, its body couldn't be read: a labelled statement.
    }

    /// `SPIRVUtils.ShaderKind` (net/vulkanmod/vulkan/shader/SPIRVUtils.java)
    pub mod spirv_utils_shader_kind {
        use super::*;
        // The class itself isn't translated: its constants carry values. Its static parts are below.
        // SPIRVUtils.ShaderKind.ShaderKind(1): not translated, its class isn't translated (its constants carry values).
    }

    /// `SPIRVUtils.ShaderReleaser` (net/vulkanmod/vulkan/shader/SPIRVUtils.java)
    pub mod spirv_utils_shader_releaser {
        use super::*;
        // The class itself isn't translated: it implements `ShadercIncludeResultReleaseI` (org.lwjgl.util.shaderc). Its static parts are below.
        // SPIRVUtils.ShaderReleaser.invoke(2): not translated, its class isn't translated (it implements `ShadercIncludeResultReleaseI` (org.lwjgl.util.shaderc)).
    }

    /// `Uniforms` (net/vulkanmod/vulkan/shader/Uniforms.java)
    pub mod uniforms {
        use super::*;
        // vec1i_uniformMap: not translated, it uses `Object2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects).
        // vec1f_uniformMap: not translated, it uses `Object2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects).
        // vec2f_uniformMap: not translated, it uses `Object2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects).
        // vec3f_uniformMap: not translated, it uses `Object2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects).
        // vec4f_uniformMap: not translated, it uses `Object2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects).
        // mat4f_uniformMap: not translated, it uses `Object2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects).
        #[derive(Clone, Debug)]
        pub struct Uniforms {
        }
        impl Uniforms {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // Uniforms.setupDefaultUniforms(0): not translated, it uses Uniforms.mat4f_uniformMap, which isn't translated (it uses `Object2ReferenceOpenHashMap` (it.unimi.dsi.fastutil.objects)).
        // Uniforms.getUniformSupplier(2): not translated, a function kept in a variable or field (only parameters can be).
    }

    /// `Synchronization` (net/vulkanmod/vulkan/Synchronization.java)
    pub mod synchronization {
        use super::*;
        // The class itself isn't translated: it uses `LongBuffer` (java.nio). Its static parts are below.
        pub const ALLOCATION_SIZE: i32 = 50;
        // INSTANCE: not translated, it uses Synchronization, which isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.Synchronization(1): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.addCommandBuffer(1): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.addCommandBuffer(2): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.addFence(1): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.waitFences(0): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.addWaitSemaphore(1): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.getWaitSemaphoreCount(0): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.getWaitSemaphores(1): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.scheduleCbReset(0): not translated, its class isn't translated (it uses `LongBuffer` (java.nio)).
        // Synchronization.waitFence(1): not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        // Synchronization.checkFenceStatus(1): not translated, it uses `VkDevice` (org.lwjgl.vulkan).
    }

    /// `SystemInfo` (net/vulkanmod/vulkan/SystemInfo.java)
    pub mod system_info {
        use super::*;
        pub static CPU_INFO: LazyLock<Option<String>> = LazyLock::new(|| None);
        #[derive(Clone, Debug)]
        pub struct SystemInfo {
        }
        impl SystemInfo {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
    }

    /// `ImageUtil` (net/vulkanmod/vulkan/texture/ImageUtil.java)
    pub mod image_util {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct ImageUtil {
        }
        impl ImageUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // ImageUtil.copyBufferToImageCmd(13): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // ImageUtil.downloadTexture(2): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // ImageUtil.copyImageToBuffer(10): not translated, it uses `Buffer` (org.lwjgl.vulkan.VkBufferImageCopy).
        // ImageUtil.copyImageToBufferCmd(12): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // ImageUtil.blitFramebuffer(2): not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // ImageUtil.blitFramebuffer(3): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // ImageUtil.blitFramebuffer(9): not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // ImageUtil.generateMipmaps(1): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // ImageUtil.imageTransferMemoryBarrier(4): not translated, it uses `MemoryStack` (org.lwjgl.system).
    }

    /// `SamplerInfo` (net/vulkanmod/vulkan/texture/SamplerInfo.java)
    pub mod sampler_info {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct SamplerInfo {
            pub encoded_state: i32,
            pub max_lod: i32,
            pub max_anisotropy: i32,
        }
        impl SamplerInfo {
            pub fn new() -> Self {
                let mut s = Self::new_9(0, 0, 0, 0, 0, 0.0f32, false, 0.0f32, -1);
                s
            }
            pub fn new_9(address_mode_u: i32, address_mode_v: i32, min_filter: i32, mag_filter: i32, mipmap_mode: i32, max_lod: f32, anisotropy: bool, max_anisotropy: f32, reduction_mode: i32) -> Self {
                let mut s = Self::new_11(address_mode_u, address_mode_v, min_filter, mag_filter, mipmap_mode, max_lod, anisotropy, max_anisotropy, false, 0, reduction_mode);
                s
            }
            pub fn new_11(address_mode_u: i32, address_mode_v: i32, min_filter: i32, mag_filter: i32, mipmap_mode: i32, max_lod: f32, anisotropy: bool, max_anisotropy: f32, compare: bool, compare_op: i32, reduction_mode: i32) -> Self {
                let mut s = Self { encoded_state: 0i32, max_lod: 0i32, max_anisotropy: 0i32 };
                s.max_lod = (max_lod as i32);
                s.max_anisotropy = (max_anisotropy as i32);
                s.encoded_state = sampler_manager::get_encoded_state(address_mode_u, address_mode_v, min_filter, mag_filter, mipmap_mode, anisotropy, compare, compare_op, reduction_mode);
                s
            }
            pub fn get_address_mode_u(&self) -> i32 {
                return (self.encoded_state >> 0) & 2;
            }
            pub fn get_address_mode_v(&self) -> i32 {
                return (self.encoded_state >> 2) & 2;
            }
            pub fn get_min_filter(&self) -> i32 {
                return (self.encoded_state >> 4) & 1;
            }
            pub fn get_mag_filter(&self) -> i32 {
                return (self.encoded_state >> 5) & 1;
            }
            pub fn get_mipmap_mode(&self) -> i32 {
                return (self.encoded_state >> 6) & 1;
            }
            pub fn get_anisotropy(&self) -> bool {
                return ((self.encoded_state >> 7) & 1) != 0;
            }
            pub fn compare_enabled(&self) -> bool {
                return ((self.encoded_state >> 8) & 1) != 0;
            }
            pub fn get_compare_op(&self) -> i32 {
                return (self.encoded_state >> 8) & 3;
            }
            pub fn has_reduction_mode(&self) -> bool {
                return ((self.encoded_state >> 12) & 1) != 0;
            }
            pub fn get_reduction_mode(&self) -> i32 {
                return (self.encoded_state >> 13) & 2;
            }
            pub fn get_max_anisotropy(&self) -> i32 {
                return self.max_anisotropy;
            }
            pub fn get_max_lod(&self) -> i32 {
                return self.max_lod;
            }
            // SamplerInfo.equals(1): not translated, it uses `Object`.
            pub fn hash_code(&self) -> i32 {
                let mut result: i32 = self.encoded_state;
                result = (31 * result) + self.max_lod;
                return (31 * result) + self.max_anisotropy;
            }
        }
        pub fn builder() -> sampler_info_builder::Builder {
            return sampler_info_builder::Builder::new();
        }
    }

    /// `SamplerInfo.Builder` (net/vulkanmod/vulkan/texture/SamplerInfo.java)
    pub mod sampler_info_builder {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Builder {
            pub address_mode_u: i32,
            pub address_mode_v: i32,
            pub min_filter: i32,
            pub mag_filter: i32,
            pub mipmap_mode: i32,
            pub max_lod: f32,
            pub anisotropy: bool,
            pub max_anisotropy: f32,
            pub compare_enabled: bool,
            pub compare_op: i32,
            pub reduction_mode: i32,
        }
        impl Builder {
            pub fn new() -> Self {
                let mut s = Self { address_mode_u: 2, address_mode_v: 2, min_filter: 1, mag_filter: 1, mipmap_mode: 1, max_lod: 0.0f32, anisotropy: false, max_anisotropy: 0.0f32, compare_enabled: false, compare_op: 7, reduction_mode: 0 };
                s
            }
            // SamplerInfo.Builder.setAddressMode(1): not translated, `this` used as a value.
            // SamplerInfo.Builder.setFiltering(3): not translated, `this` used as a value.
            // SamplerInfo.Builder.setMaxLod(1): not translated, `this` used as a value.
            // SamplerInfo.Builder.setAnisotropy(1): not translated, `this` used as a value.
            // SamplerInfo.Builder.setCompare(2): not translated, `this` used as a value.
            // SamplerInfo.Builder.setReductionMode(1): not translated, `this` used as a value.
            pub fn create_sampler_info(&self) -> sampler_info::SamplerInfo {
                return sampler_info::SamplerInfo::new_11(self.address_mode_u, self.address_mode_v, self.min_filter, self.mag_filter, self.mipmap_mode, self.max_lod, self.anisotropy, self.max_anisotropy, self.compare_enabled, self.compare_op, self.reduction_mode);
            }
        }
    }

    /// `SamplerManager` (net/vulkanmod/vulkan/texture/SamplerManager.java)
    pub mod sampler_manager {
        use super::*;
        pub const ADDRESS_MODE_BITS: i32 = 2;
        pub const REDUCTION_MODE_BITS: i32 = 2;
        pub const COMPARE_OP_BITS: i32 = 3;
        pub const ADDRESS_MODE_U_OFFSET: i32 = 0;
        pub const ADDRESS_MODE_V_OFFSET: i32 = 2;
        pub const MIN_FILTER_OFFSET: i32 = 4;
        pub const MAG_FILTER_OFFSET: i32 = 5;
        pub const MIPMAP_MODE_OFFSET: i32 = 6;
        pub const ANISOTROPY_OFFSET: i32 = 7;
        pub const COMPARE_ENABLED_OFFSET: i32 = 8;
        pub const COMPARE_OP_OFFSET: i32 = 9;
        pub const REDUCTION_MODE_ENABLE_OFFSET: i32 = 12;
        pub const REDUCTION_MODE_OFFSET: i32 = 13;
        pub const MIP_BIAS: f32 = -0.5f32;
        // SAMPLERS: not translated, it uses `Object2LongMap` (it.unimi.dsi.fastutil.objects).
        #[derive(Clone, Debug)]
        pub struct SamplerManager {
        }
        impl SamplerManager {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // SamplerManager.getSampler(3): not translated, it calls SamplerManager.getSampler, which isn't translated.
        // SamplerManager.getSampler(5): not translated, it calls SamplerManager.getSampler, which isn't translated.
        // SamplerManager.getSampler(9): not translated, it calls SamplerManager.getSampler, which isn't translated.
        // SamplerManager.getSampler(1): not translated, it uses SamplerManager.SAMPLERS, which isn't translated (it uses `Object2LongMap` (it.unimi.dsi.fastutil.objects)).
        // SamplerManager.getDefaultSampler(0): not translated, it calls SamplerManager.getSampler, which isn't translated.
        // SamplerManager.createTextureSampler(1): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // SamplerManager.cleanUp(0): not translated, it uses `LongIterator` (it.unimi.dsi.fastutil.longs).
        pub fn get_encoded_state(address_mode_u: i32, address_mode_v: i32, min_filter: i32, mag_filter: i32, mipmap_mode: i32, anisotropy: bool, compare: bool, compare_op: i32, reduction_mode: i32) -> i32 {
            let mut encoded_state: i32 = (address_mode_u & 2) << 0;
            encoded_state |= (address_mode_v & 2) << 2;
            encoded_state |= (min_filter & 1) << 4;
            encoded_state |= (mag_filter & 1) << 5;
            encoded_state |= (mipmap_mode & 1) << 6;
            encoded_state |= ((if anisotropy { 1 } else { 0 }) & 1) << 7;
            encoded_state |= ((if compare { 1 } else { 0 }) & 1) << 8;
            encoded_state |= (compare_op & 3) << 9;
            encoded_state |= (if reduction_mode != (-1) { 1 } else { 0 }) << 12;
            return encoded_state | ((reduction_mode & 2) << 13);
        }
    }

    /// `VTextureSelector` (net/vulkanmod/vulkan/texture/VTextureSelector.java)
    pub mod v_texture_selector {
        use super::*;
        pub const SIZE: i32 = 12;
        // boundTextures: not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        pub static LEVELS: LazyLock<Vec<i32>> = LazyLock::new(|| vec![0i32; 12]);
        // whiteTexture: not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        pub static ACTIVE_TEXTURE: JCell<i32> = JCell::new(0);
        #[derive(Clone, Debug)]
        pub struct VTextureSelector {
        }
        impl VTextureSelector {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // VTextureSelector.bindTexture(1): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        // VTextureSelector.bindTexture(2): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        // VTextureSelector.bindImage(3): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        // VTextureSelector.uploadSubTexture(9): not translated, it uses `ByteBuffer` (java.nio).
        // VTextureSelector.uploadSubTexture(10): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        pub fn get_texture_idx(name: &str) -> i32 {
            return match &*name {
                "Sampler0" | "DiffuseSampler" | "InSampler" | "CloudFaces" | "Sprite" | "CurrentSprite" => 0,
                "Sampler1" | "BlurSampler" | "NextSprite" => 1,
                "Sampler2" => 2,
                "Sampler3" => 3,
                "Sampler4" => 4,
                "Sampler5" => 5,
                "Sampler6" => 6,
                "Sampler7" => 7,
                _ => -1,
            };
        }
        // VTextureSelector.bindShaderTextures(1): not translated, it uses Pipeline, which isn't translated (it uses DescriptorSets, which isn't translated (it uses `IntBuffer` (java.nio))).
        // VTextureSelector.getImage(1): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        // VTextureSelector.setLightTexture(1): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        // VTextureSelector.setOverlayTexture(1): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        pub fn set_active_texture(active_texture: i32) {
            if (active_texture < 0) || (active_texture >= 12) {
                // log:
            }
            v_texture_selector::ACTIVE_TEXTURE.set(active_texture);
        }
        // VTextureSelector.getBoundTexture(0): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        // VTextureSelector.getBoundTexture(1): not translated, it uses VTextureSelector.boundTextures, which isn't translated (a VulkanImage with no value (null in Java)).
        // VTextureSelector.getWhiteTexture(0): not translated, it uses VTextureSelector.whiteTexture, which isn't translated (its value reads other changing statics).
    }

    /// `VulkanImage` (net/vulkanmod/vulkan/texture/VulkanImage.java)
    pub mod vulkan_image {
        use super::*;
        // The class itself isn't translated: it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints). Its static parts are below.
        pub static DEFAULT_FORMAT: JCell<i32> = JCell::new(37);
        // DEVICE: not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        // VulkanImage.VulkanImage(9): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.VulkanImage(1): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.createTextureImage(1): not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.createDepthImage(6): not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.createWhiteTexture(0): not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.createImage(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        pub fn get_aspect(format: i32) -> i32 {
            return match format {
                124 | 125 | 126 => 2,
                _ => 1,
                129 | 130 => 6,
            };
        }
        pub fn is_depth_format(format: i32) -> bool {
            return match format {
                124 | 125 | 126 | 129 | 130 => true,
                _ => false,
            };
        }
        // VulkanImage.createImageView(5): not translated, it calls VulkanImage.createImageView, which isn't translated.
        // VulkanImage.createImageView(7): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // VulkanImage.uploadSubTextureAsync(9): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.uploadSubTextureAsync(10): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.uploadSubTextureAsync(10): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.transferDstLayout(2): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.readOnlyLayout(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.readOnlyLayout(2): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.setSampler(1): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.transitionImageLayout(3): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.transitionImageLayout(4): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // VulkanImage.transitionLayout(9): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // VulkanImage.transitionLayout(10): not translated, it uses `MemoryStack` (org.lwjgl.system).
        pub fn has_stencil_component(format: i32) -> bool {
            return (format == 130) || (format == 129);
        }
        // VulkanImage.free(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.doFree(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.getCurrentLayout(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.setCurrentLayout(1): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.getId(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.getAllocation(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.getImageView(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.getImageView(1): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.getLevelImageView(1): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.getLevelImageViews(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        // VulkanImage.getSampler(0): not translated, its class isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        pub fn builder(width: i32, height: i32) -> vulkan_image_builder::Builder {
            return vulkan_image_builder::Builder::new(width, height);
        }
    }

    /// `VulkanImage.Builder` (net/vulkanmod/vulkan/texture/VulkanImage.java)
    pub mod vulkan_image_builder {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct Builder {
            pub width: i32,
            pub height: i32,
            pub name: Option<String>,
            pub format: i32,
            pub format_size: i32,
            pub array_layers: i32,
            pub mip_levels: i8,
            pub usage: i32,
            pub view_type: i32,
            pub linear_filtering: bool,
            pub clamp: bool,
            pub reduction_mode: i32,
        }
        impl Builder {
            pub fn new(width: i32, height: i32) -> Self {
                let mut s = Self { width: width, height: height, name: None, format: vulkan_image::DEFAULT_FORMAT.get(), format_size: 0i32, array_layers: 1, mip_levels: 1i8, usage: 7, view_type: 1, linear_filtering: false, clamp: false, reduction_mode: -1 };
                s.width = width;
                s.height = height;
                s
            }
            // VulkanImage.Builder.setName(1): not translated, `this` used as a value.
            // VulkanImage.Builder.setFormat(1): not translated, `this` used as a value.
            // VulkanImage.Builder.setArrayLayers(1): not translated, `this` used as a value.
            // VulkanImage.Builder.setMipLevels(1): not translated, `this` used as a value.
            // VulkanImage.Builder.setUsage(1): not translated, `this` used as a value.
            // VulkanImage.Builder.addUsage(1): not translated, `this` used as a value.
            // VulkanImage.Builder.setViewType(1): not translated, `this` used as a value.
            // VulkanImage.Builder.setLinearFiltering(1): not translated, `this` used as a value.
            // VulkanImage.Builder.setClamp(1): not translated, `this` used as a value.
            // VulkanImage.Builder.setSamplerReductionMode(1): not translated, `this` used as a value.
            // VulkanImage.Builder.createVulkanImage(0): not translated, it uses VulkanImage, which isn't translated (it uses `Int2LongArrayMap` (it.unimi.dsi.fastutil.ints)).
        }
        pub fn format_size(format: i32) -> i32 {
            return match format {
                9 => 1,
                37 | 41 | 42 | 43 | 100 | 126 | 129 => 4,
                76 => 2,
                97 => 8,
                _ => {
                panic!("{}", format!("Unxepcted format: {}", format));
                },
            };
        }
    }

    /// `ColorUtil` (net/vulkanmod/vulkan/util/ColorUtil.java)
    pub mod color_util {
        use super::*;
        pub const COLOR_INV: f32 = 0.003921569f32;
        // colorConsumer: not translated, it uses ColorConsumer, which isn't translated (it's an interface).
        #[derive(Clone, Debug)]
        pub struct ColorUtil {
        }
        impl ColorUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // ColorUtil.useGammaCorrection(1): not translated, it uses ColorUtil.colorConsumer, which isn't translated (it uses ColorConsumer, which isn't translated (it's an interface)).
        pub fn float_to_int(f: f32) -> i32 {
            return ((f * 255.0f32) as i32) & 0xFF;
        }
        pub fn unpack_color(c: i32, s: i32) -> f32 {
            return (((c >> s) & 0xFF) as f32) * 0.003921569f32;
        }
        pub fn bgr_ato_rgba(v: i32) -> i32 {
            let r: i8 = ((v >> 16) as i8);
            let b: i8 = (v as i8);
            return (((r as i32) & 0xFF) | (((b as i32) << 16) & 0xFF0000)) | (v & (-16711936));
        }
        pub fn gamma(f: f32) -> f32 {
            return (((f as f64).powf(2.2f64)) as f32);
        }
        // ColorUtil.setRGBA_Buffer(5): not translated, it's overloaded with the same number of parameters.
        // ColorUtil.setRGBA_Buffer(5): not translated, it's overloaded with the same number of parameters.
    }

    /// `ColorUtil.ARGB` (net/vulkanmod/vulkan/util/ColorUtil.java)
    pub mod color_util_argb {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct ARGB {
        }
        impl ARGB {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn pack(r: f32, g: f32, b: f32, a: f32) -> i32 {
            return (((color_util::float_to_int(a) << 24) | (color_util::float_to_int(r) << 16)) | (color_util::float_to_int(g) << 8)) | color_util::float_to_int(b);
        }
        pub fn unpack_r(color: i32) -> f32 {
            return color_util::unpack_color(color, 16);
        }
        pub fn unpack_g(color: i32) -> f32 {
            return color_util::unpack_color(color, 8);
        }
        pub fn unpack_b(color: i32) -> f32 {
            return color_util::unpack_color(color, 0);
        }
        pub fn unpack_a(color: i32) -> f32 {
            return color_util::unpack_color(color, 24);
        }
        pub fn multiply_alpha(color: i32, m: f32) -> i32 {
            let new_a: i32 = color_util::float_to_int(color_util_argb::unpack_a(color) * m);
            return (color & 16777215) | (new_a << 24);
        }
        pub fn multiply_rgb(color: i32, m: f32) -> i32 {
            let alpha: i32 = (((color as u32) >> 24) as i32) & 0xFF;
            let red: i32 = (((((((color as u32) >> 16) as i32) & 0xFF) as f32) * m) as i32);
            let green: i32 = (((((((color as u32) >> 8) as i32) & 0xFF) as f32) * m) as i32);
            let blue: i32 = ((((color & 0xFF) as f32) * m) as i32);
            return (((alpha << 24) | (red << 16)) | (green << 8)) | blue;
        }
        pub fn to_rgba(color: i32) -> i32 {
            return ((color & (-16711936)) | ((color >> 16) & 0xFF)) | ((color << 16) & 0xFF0000);
        }
    }

    /// `ColorUtil.ColorConsumer` (net/vulkanmod/vulkan/util/ColorUtil.java)
    pub mod color_util_color_consumer {
        use super::*;
        // The class itself isn't translated: it's an interface. Its static parts are below.
        // ColorUtil.ColorConsumer.setRGBA_Buffer(5): not translated, it has no body.
        // ColorUtil.ColorConsumer.setRGBA_Buffer(5): not translated, it has no body.
        // ColorUtil.ColorConsumer.setRGBA_Buffer(5): not translated, it has no body.
        // ColorUtil.ColorConsumer.putColor(5): not translated, its class isn't translated (it's an interface).
        // ColorUtil.ColorConsumer.putColor(5): not translated, its class isn't translated (it's an interface).
        // ColorUtil.ColorConsumer.putColor(5): not translated, its class isn't translated (it's an interface).
    }

    /// `ColorUtil.DefaultColorConsumer` (net/vulkanmod/vulkan/util/ColorUtil.java)
    pub mod color_util_default_color_consumer {
        use super::*;
        // The class itself isn't translated: it implements `ColorConsumer`. Its static parts are below.
        // ColorUtil.DefaultColorConsumer.setRGBA_Buffer(5): not translated, its class isn't translated (it implements `ColorConsumer`).
        // ColorUtil.DefaultColorConsumer.setRGBA_Buffer(5): not translated, its class isn't translated (it implements `ColorConsumer`).
        // ColorUtil.DefaultColorConsumer.setRGBA_Buffer(5): not translated, its class isn't translated (it implements `ColorConsumer`).
    }

    /// `ColorUtil.GammaColorConsumer` (net/vulkanmod/vulkan/util/ColorUtil.java)
    pub mod color_util_gamma_color_consumer {
        use super::*;
        // The class itself isn't translated: it implements `ColorConsumer`. Its static parts are below.
        // ColorUtil.GammaColorConsumer.setRGBA_Buffer(5): not translated, its class isn't translated (it implements `ColorConsumer`).
        // ColorUtil.GammaColorConsumer.setRGBA_Buffer(5): not translated, its class isn't translated (it implements `ColorConsumer`).
        // ColorUtil.GammaColorConsumer.setRGBA_Buffer(5): not translated, its class isn't translated (it implements `ColorConsumer`).
    }

    /// `ColorUtil.RGBA` (net/vulkanmod/vulkan/util/ColorUtil.java)
    pub mod color_util_rgba {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct RGBA {
        }
        impl RGBA {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn pack(r: f32, g: f32, b: f32, a: f32) -> i32 {
            return (((color_util::float_to_int(a) << 24) | (color_util::float_to_int(b) << 16)) | (color_util::float_to_int(g) << 8)) | color_util::float_to_int(r);
        }
        pub fn unpack_r(color: i32) -> f32 {
            return color_util::unpack_color(color, 24);
        }
        pub fn unpack_g(color: i32) -> f32 {
            return color_util::unpack_color(color, 16);
        }
        pub fn unpack_b(color: i32) -> f32 {
            return color_util::unpack_color(color, 8);
        }
        pub fn from_argb32(i: i32) -> i32 {
            return ((i & (-16711936)) | ((i & 0xFF0000) >> 16)) | ((i & 0xFF) << 16);
        }
    }

    /// `DrawUtil` (net/vulkanmod/vulkan/util/DrawUtil.java)
    pub mod draw_util_2 {
        use super::*;
        // The class itself isn't translated: another class has the same name, in a different package. Its static parts are below.
        // DrawUtil.blitToScreen(0): not translated, it calls DrawUtil.fastBlit, which isn't translated.
        // DrawUtil.fastBlit(0): not translated, it uses GraphicsPipeline, which isn't translated (it extends `Pipeline`).
    }

    /// `MappedBuffer` (net/vulkanmod/vulkan/util/MappedBuffer.java)
    pub mod mapped_buffer {
        use super::*;
        // The class itself isn't translated: it uses `ByteBuffer` (java.nio). Its static parts are below.
        // MappedBuffer.createFromBuffer(1): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // MappedBuffer.MappedBuffer(2): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // MappedBuffer.MappedBuffer(1): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // MappedBuffer.putFloat(2): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // MappedBuffer.putInt(2): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // MappedBuffer.getFloat(1): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
        // MappedBuffer.getInt(1): not translated, its class isn't translated (it uses `ByteBuffer` (java.nio)).
    }

    /// `MemoryAccess` (net/vulkanmod/vulkan/util/MemoryAccess.java)
    pub mod memory_access {
        use super::*;
        // UNSAFE: not translated, it uses `Unsafe` (sun.misc).
        // BITS64: not translated, it uses `Pointer` (org.lwjgl.system).BITS64.
        #[derive(Clone, Debug)]
        pub struct MemoryAccess {
        }
        impl MemoryAccess {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // MemoryAccess.memGetBoolean(1): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memGetByte(1): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memGetShort(1): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memGetInt(1): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memGetLong(1): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memGetFloat(1): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memGetDouble(1): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memGetAddress(1): not translated, it uses MemoryAccess.BITS64, which isn't translated (it uses `Pointer` (org.lwjgl.system).BITS64).
        // MemoryAccess.memPutByte(2): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memPutShort(2): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memPutInt(2): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memPutLong(2): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memPutFloat(2): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
        // MemoryAccess.memPutDouble(2): not translated, it uses MemoryAccess.UNSAFE, which isn't translated (it uses `Unsafe` (sun.misc)).
    }

    /// `Pair` (net/vulkanmod/vulkan/util/Pair.java)
    pub mod pair {
        use super::*;
        // The class itself isn't translated: it's generic. Its static parts are below.
        // Pair.Pair(2): not translated, its class isn't translated (it's generic).
    }

    /// `ScreenshotUtil` (net/vulkanmod/vulkan/util/ScreenshotUtil.java)
    pub mod screenshot_util {
        use super::*;
        #[derive(Clone, Debug)]
        pub struct ScreenshotUtil {
        }
        impl ScreenshotUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // ScreenshotUtil.takeScreenshot(3): not translated, it uses Minecraft's `RenderTarget`.
    }

    /// `VkResult` (net/vulkanmod/vulkan/util/VkResult.java)
    pub mod vk_result {
        use super::*;
        pub const VK_SUCCESS: i32 = 0;
        pub const VK_NOT_READY: i32 = 1;
        pub const VK_TIMEOUT: i32 = 2;
        pub const VK_EVENT_SET: i32 = 3;
        pub const VK_EVENT_RESET: i32 = 4;
        pub const VK_INCOMPLETE: i32 = 5;
        pub const VK_ERROR_OUT_OF_HOST_MEMORY: i32 = -1;
        pub const VK_ERROR_OUT_OF_DEVICE_MEMORY: i32 = -2;
        pub const VK_ERROR_INITIALIZATION_FAILED: i32 = -3;
        pub const VK_ERROR_DEVICE_LOST: i32 = -4;
        pub const VK_ERROR_MEMORY_MAP_FAILED: i32 = -5;
        pub const VK_ERROR_LAYER_NOT_PRESENT: i32 = -6;
        pub const VK_ERROR_EXTENSION_NOT_PRESENT: i32 = -7;
        pub const VK_ERROR_FEATURE_NOT_PRESENT: i32 = -8;
        pub const VK_ERROR_INCOMPATIBLE_DRIVER: i32 = -9;
        pub const VK_ERROR_TOO_MANY_OBJECTS: i32 = -10;
        pub const VK_ERROR_FORMAT_NOT_SUPPORTED: i32 = -11;
        pub const VK_ERROR_FRAGMENTED_POOL: i32 = -12;
        pub const VK_ERROR_UNKNOWN: i32 = -13;
        #[derive(Clone, Debug)]
        pub struct VkResult {
        }
        impl VkResult {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn decode(result: i32) -> String {
            return match result {
                -13 => "VK_ERROR_UNKNOWN".to_string(),
                -12 => "VK_ERROR_FRAGMENTED_POOL".to_string(),
                -11 => "VK_ERROR_FORMAT_NOT_SUPPORTED".to_string(),
                -10 => "VK_ERROR_TOO_MANY_OBJECTS".to_string(),
                -9 => "VK_ERROR_INCOMPATIBLE_DRIVER".to_string(),
                -8 => "VK_ERROR_FEATURE_NOT_PRESENT".to_string(),
                -7 => "VK_ERROR_EXTENSION_NOT_PRESENT".to_string(),
                -6 => "VK_ERROR_LAYER_NOT_PRESENT".to_string(),
                -5 => "VK_ERROR_MEMORY_MAP_FAILED".to_string(),
                -4 => "VK_ERROR_DEVICE_LOST".to_string(),
                -3 => "VK_ERROR_INITIALIZATION_FAILED".to_string(),
                -2 => "VK_ERROR_OUT_OF_DEVICE_MEMORY".to_string(),
                -1 => "VK_ERROR_OUT_OF_HOST_MEMORY".to_string(),
                0 => "VK_SUCCESS".to_string(),
                1 => "VK_NOT_READY".to_string(),
                2 => "VK_TIMEOUT".to_string(),
                3 => "VK_EVENT_SET".to_string(),
                4 => "VK_EVENT_RESET".to_string(),
                5 => "VK_INCOMPLETE".to_string(),
                _ => format!("{}", result),
            };
        }
    }

    /// `VUtil` (net/vulkanmod/vulkan/util/VUtil.java)
    pub mod v_util {
        use super::*;
        pub const CHECKS: bool = true;
        pub const UINT32_MAX: i32 = -1;
        pub const UINT64_MAX: i64 = -1i64;
        // UNSAFE: not translated, it uses `Unsafe` (sun.misc).
        #[derive(Clone, Debug)]
        pub struct VUtil {
        }
        impl VUtil {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // VUtil.asPointerBuffer(1): not translated, it uses `PointerBuffer` (org.lwjgl).
        // VUtil.memcpy(2): not translated, it uses `ByteBuffer` (java.nio).
        // VUtil.memcpy(3): not translated, it's overloaded with the same number of parameters.
        // VUtil.memcpy(3): not translated, it's overloaded with the same number of parameters.
        // VUtil.memcpy(5): not translated, it uses `ByteBuffer` (java.nio).
        pub fn align(x: i32, align: i32) -> i32 {
            let r: i32 = x % align;
            return if r == 0 { x } else { (x + align) - r };
        }
    }

    /// `VRenderSystem` (net/vulkanmod/vulkan/VRenderSystem.java)
    pub mod v_render_system {
        use super::*;
        pub const DEFAULT_DEPTH_VALUE: f32 = 1.0f32;
        pub static WINDOW: JCell<i64> = JCell::new(0i64);
        pub static DEPTH_TEST: JCell<bool> = JCell::new(true);
        pub static DEPTH_MASK: JCell<bool> = JCell::new(true);
        pub static DEPTH_FUN: JCell<i32> = JCell::new(515);
        pub static TOPOLOGY: JCell<i32> = JCell::new(3);
        pub static POLYGON_MODE: JCell<i32> = JCell::new(0);
        pub static CAN_SET_LINE_WIDTH: JCell<bool> = JCell::new(false);
        pub static COLOR_MASK: LazyLock<JCell<i32>> = LazyLock::new(|| JCell::new(pipeline_state_color_mask::get_color_mask(true, true, true, true)));
        pub static CULL: JCell<bool> = JCell::new(true);
        pub static LOGIC_OP: JCell<bool> = JCell::new(false);
        pub static LOGIC_OP_FUN: JCell<i32> = JCell::new(0);
        // shaderTextures: not translated, it uses Minecraft's `GpuTextureView`.
        pub static CLEAR_DEPTH_VALUE: JCell<f32> = JCell::new(1.0f32);
        // clearColor: not translated, it uses `FloatBuffer` (java.nio).
        // modelViewMatrix: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // projectionMatrix: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // TextureMatrix: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // MVP: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // modelOffset: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // lightDirection0: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // lightDirection1: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // shaderColor: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // shaderFogColor: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // fogData: not translated, it uses Minecraft's `FogData`.
        // screenSize: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // textureSize: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // texelSize: not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        pub static ALPHA_CUTOUT: JCell<f32> = JCell::new(0.0f32);
        pub static DEPTH_BIAS_ENABLED: JCell<bool> = JCell::new(false);
        pub static DEPTH_BIAS_CONSTANT: JCell<f32> = JCell::new(0.0f32);
        pub static DEPTH_BIAS_SLOPE: JCell<f32> = JCell::new(0.0f32);
        pub static PROJECTION: LazyLock<JCell<Option<JMat4>>> = LazyLock::new(|| JCell::new(None));
        pub static MODEL_VIEW: LazyLock<JCell<Option<JMat4>>> = LazyLock::new(|| JCell::new(None));
        pub static CURRENT_TIME: JCell<i32> = JCell::new(0i32);
        #[derive(Clone, Debug)]
        pub struct VRenderSystem {
        }
        impl VRenderSystem {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        // VRenderSystem.initRenderer(0): not translated, it calls Vulkan.initVulkan, which isn't translated.
        // VRenderSystem.setShaderTexture(2): not translated, it uses Minecraft's `GpuTextureView`.
        // VRenderSystem.getShaderTexture(1): not translated, it uses Minecraft's `GpuTextureView`.
        // VRenderSystem.getScreenSize(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.updateScreenSize(0): not translated, it uses Minecraft's `Window`.
        // VRenderSystem.getTextureSize(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.getTexelSize(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.setTextureSize(2): not translated, it uses VRenderSystem.textureSize, which isn't translated (it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio))).
        pub fn set_window(window: i64) {
            v_render_system::WINDOW.set(window);
        }
        // VRenderSystem.getModelOffset(0): not translated, it uses `ByteBuffer` (java.nio).
        // VRenderSystem.maxSupportedTextureSize(0): not translated, it uses DeviceManager.deviceProperties, which isn't translated (it uses `VkPhysicalDeviceProperties` (org.lwjgl.vulkan)).
        // VRenderSystem.applyMVP(2): not translated, it's overloaded with the same number of parameters.
        // VRenderSystem.applyModelViewMatrix(1): not translated, JOML's Matrix4f.get.
        // VRenderSystem.applyProjectionMatrix(1): not translated, it's overloaded with the same number of parameters.
        // VRenderSystem.applyProjectionMatrix(1): not translated, it's overloaded with the same number of parameters.
        // VRenderSystem.calculateMVP(0): not translated, it uses VRenderSystem.modelViewMatrix, which isn't translated (it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio))).
        // VRenderSystem.setTextureMatrix(1): not translated, JOML's Matrix4f.get.
        // VRenderSystem.getTextureMatrix(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.getModelViewMatrix(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.getProjectionMatrix(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.getMVP(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.setModelOffset(3): not translated, it uses VRenderSystem.modelOffset, which isn't translated (it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio))).
        // VRenderSystem.setShaderColor(4): not translated, it calls ColorUtil.setRGBA_Buffer, which isn't translated.
        // VRenderSystem.setShaderFogColor(4): not translated, it calls ColorUtil.setRGBA_Buffer, which isn't translated.
        // VRenderSystem.getShaderColor(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.getShaderFogColor(0): not translated, it uses MappedBuffer, which isn't translated (it uses `ByteBuffer` (java.nio)).
        // VRenderSystem.getFogData(0): not translated, it uses Minecraft's `FogData`.
        pub fn set_current_time(current_time: i32) {
            v_render_system::CURRENT_TIME.set(current_time);
        }
        pub fn get_current_time() -> i32 {
            return v_render_system::CURRENT_TIME.get();
        }
        // VRenderSystem.setClearColor(4): not translated, it calls ColorUtil.setRGBA_Buffer, which isn't translated.
        // VRenderSystem.clear(1): not translated, it calls Renderer.clearAttachments, which isn't translated.
        pub fn clear_depth(depth: f64) {
            v_render_system::CLEAR_DEPTH_VALUE.set((depth as f32));
        }
        pub fn disable_depth_test() {
            v_render_system::DEPTH_TEST.set(false);
        }
        pub fn depth_mask(b: bool) {
            v_render_system::DEPTH_MASK.set(b);
        }
        pub fn set_primitive_topology_gl(mode: i32) {
            v_render_system::TOPOLOGY.set(match mode {
                0 => 0,
                1 | 3 => 1,
                _ => {
                panic!("{}", format!("Unknown GL primitive topology: {}", mode));
                },
                4 | 5 | 6 => 3,
            });
        }
        pub fn set_polygon_mode_gl(mode: i32) {
            v_render_system::POLYGON_MODE.set(match mode {
                6912 => 2,
                6913 => 1,
                6914 => 0,
                _ => {
                panic!("{}", format!("Unknown GL polygon mode: {}", mode));
                },
            });
        }
        // VRenderSystem.setLineWidth(1): not translated, it calls Renderer.setLineWidth, which isn't translated.
        pub fn color_mask_4(b: bool, b1: bool, b2: bool, b3: bool) {
            v_render_system::COLOR_MASK.set(pipeline_state_color_mask::get_color_mask(b, b1, b2, b3));
        }
        pub fn color_mask_1(write_mask: i32) {
            v_render_system::COLOR_MASK.set(pipeline_state_color_mask::get_color_mask((write_mask & 1) != 0, (write_mask & 2) != 0, (write_mask & 4) != 0, (write_mask & 8) != 0));
        }
        pub fn get_color_mask() -> i32 {
            return v_render_system::COLOR_MASK.get();
        }
        pub fn enable_depth_test() {
            v_render_system::DEPTH_TEST.set(true);
        }
        pub fn enable_cull() {
            v_render_system::CULL.set(true);
        }
        pub fn disable_cull() {
            v_render_system::CULL.set(false);
        }
        pub fn depth_func(depth_fun: i32) {
            v_render_system::DEPTH_FUN.set(depth_fun);
        }
        pub fn enable_blend() {
            let value_0 = true;
            pipeline_state::BLEND_INFO.with(|o| o.enabled = value_0);
        }
        pub fn disable_blend() {
            let value_0 = false;
            pipeline_state::BLEND_INFO.with(|o| o.enabled = value_0);
        }
        pub fn blend_func(src_factor: i32, dst_factor: i32) {
            { let args_0 = (src_factor, dst_factor); pipeline_state::BLEND_INFO.with(|o| o.set_blend_function(args_0.0, args_0.1)) };
        }
        pub fn blend_func_separate(src_factor_rgb: i32, dst_factor_rgb: i32, src_factor_alpha: i32, dst_factor_alpha: i32) {
            { let args_0 = (src_factor_rgb, dst_factor_rgb, src_factor_alpha, dst_factor_alpha); pipeline_state::BLEND_INFO.with(|o| o.set_blend_func_separate(args_0.0, args_0.1, args_0.2, args_0.3)) };
        }
        pub fn blend_op(op: i32) {
            { let args_0 = (op,); pipeline_state::BLEND_INFO.with(|o| o.set_blend_op(args_0.0)) };
        }
        pub fn enable_color_logic_op() {
            v_render_system::LOGIC_OP.set(true);
        }
        pub fn disable_color_logic_op() {
            v_render_system::LOGIC_OP.set(false);
        }
        pub fn logic_op(gl_logic_op: i32) {
            v_render_system::LOGIC_OP_FUN.set(gl_logic_op);
        }
        // VRenderSystem.polygonOffset(2): not translated, it calls Renderer.setDepthBias, which isn't translated.
        // VRenderSystem.enablePolygonOffset(0): not translated, it calls Renderer.setDepthBias, which isn't translated.
        // VRenderSystem.disablePolygonOffset(0): not translated, it calls Renderer.setDepthBias, which isn't translated.
    }

    /// `Vulkan` (net/vulkanmod/vulkan/Vulkan.java)
    pub mod vulkan {
        use super::*;
        pub const ENABLE_VALIDATION_LAYERS: bool = false;
        pub const DYNAMIC_RENDERING: bool = true;
        pub static VALIDATION_LAYERS: LazyLock<Option<JSet<String>>> = LazyLock::new(|| None);
        pub static REQUIRED_EXTENSION: LazyLock<JSet<String>> = LazyLock::new(|| vulkan::get_required_extension_set());
        pub static WINDOW: JCell<i64> = JCell::new(0i64);
        pub static INSTANCE_EXTENSIONS: LazyLock<JCell<Option<JSet<String>>>> = LazyLock::new(|| JCell::new(None));
        // instance: not translated, it uses `VkInstance` (org.lwjgl.vulkan).
        pub static DEBUG_MESSENGER: JCell<i64> = JCell::new(0i64);
        pub static SURFACE: JCell<i64> = JCell::new(0i64);
        pub static COMMAND_POOL: JCell<i64> = JCell::new(0i64);
        // immediateCmdBuffer: not translated, it uses `VkCommandBuffer` (org.lwjgl.vulkan).
        pub static IMMEDIATE_FENCE: JCell<i64> = JCell::new(0i64);
        pub static ALLOCATOR: JCell<i64> = JCell::new(0i64);
        // stagingBuffers: not translated, it uses StagingBuffers, which isn't translated (it uses `Stack` (it.unimi.dsi.fastutil)).
        pub static USE24_BITS_DEPTH_FORMAT: JCell<bool> = JCell::new(true);
        pub static DEFAULT_DEPTH_FORMAT: JCell<i32> = JCell::new(0);
        #[derive(Clone, Debug)]
        pub struct Vulkan {
        }
        impl Vulkan {
            pub fn new() -> Self {
                let mut s = Self {};
                s
            }
        }
        pub fn get_required_extension_set() -> JSet<String> {
            let mut extensions: Vec<String> = (vec!["VK_KHR_swapchain".to_string()]).to_vec();
            extensions.push("VK_KHR_dynamic_rendering".to_string());
            return JSet::from_iter(extensions.iter().cloned());
        }
        // Vulkan.debugCallback(4): not translated, it uses `VkDebugUtilsMessengerCallbackDataEXT` (org.lwjgl.vulkan).
        // Vulkan.createDebugUtilsMessengerEXT(4): not translated, it uses `VkInstance` (org.lwjgl.vulkan).
        // Vulkan.destroyDebugUtilsMessengerEXT(3): not translated, it uses `VkInstance` (org.lwjgl.vulkan).
        // Vulkan.initVulkan(1): not translated, it calls DeviceManager.init, which isn't translated.
        // Vulkan.createStagingBuffers(0): not translated, it uses Vulkan.stagingBuffers, which isn't translated (it uses StagingBuffers, which isn't translated (it uses `Stack` (it.unimi.dsi.fastutil))).
        // Vulkan.setupDepthFormat(0): not translated, it calls DeviceManager.findDepthFormat, which isn't translated.
        // Vulkan.waitIdle(0): not translated, it uses DeviceManager.vkDevice, which isn't translated (it uses `VkDevice` (org.lwjgl.vulkan)).
        // Vulkan.cleanUp(0): not translated, it uses DeviceManager.vkDevice, which isn't translated (it uses `VkDevice` (org.lwjgl.vulkan)).
        // Vulkan.freeStagingBuffers(0): not translated, it uses Vulkan.stagingBuffers, which isn't translated (it uses StagingBuffers, which isn't translated (it uses `Stack` (it.unimi.dsi.fastutil))).
        // Vulkan.createInstance(0): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Vulkan.checkValidationLayerSupport(0): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Vulkan.populateDebugMessengerCreateInfo(1): not translated, it uses `VkDebugUtilsMessengerCreateInfoEXT` (org.lwjgl.vulkan).
        pub fn setup_debug_messenger() {
        }
        // Vulkan.setDebugLabel(4): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Vulkan.createSurface(1): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Vulkan.createVma(0): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Vulkan.createCommandPool(0): not translated, it uses `MemoryStack` (org.lwjgl.system).
        // Vulkan.getRequiredInstanceExtensions(0): not translated, it uses `PointerBuffer` (org.lwjgl).
        // Vulkan.querySupportedInstanceExtension(0): not translated, it uses `MemoryStack` (org.lwjgl.system).
        pub fn check_result(result: i32, error_message: &str) {
            if result != 0 {
                panic!("{}", format!("{}: {}", error_message, vk_result::decode(result)));
            }
        }
        // Vulkan.setVsync(1): not translated, it uses SwapChain, which isn't translated (it extends `Framebuffer`).
        // Vulkan.getVkDevice(0): not translated, it uses `VkDevice` (org.lwjgl.vulkan).
        pub fn get_allocator() -> i64 {
            return vulkan::ALLOCATOR.get();
        }
        pub fn get_default_depth_format() -> i32 {
            return vulkan::DEFAULT_DEPTH_FORMAT.get();
        }
        pub fn get_surface() -> i64 {
            return vulkan::SURFACE.get();
        }
        pub fn get_command_pool() -> i64 {
            return vulkan::COMMAND_POOL.get();
        }
        // Vulkan.getStagingBuffer(0): not translated, it uses StagingBuffer, which isn't translated (it extends `Buffer`).
        // Vulkan.getStagingBuffers(0): not translated, it uses StagingBuffers, which isn't translated (it uses `Stack` (it.unimi.dsi.fastutil)).
        // Vulkan.getDevice(0): not translated, it uses Device, which isn't translated (it uses `VkPhysicalDevice` (org.lwjgl.vulkan)).
    }

    /// JOML's Vector3f as a value (Java's methods change the vector; the translation writes the result back).
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct JVec3 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }
    impl JVec3 {
        pub const fn new(x: f32, y: f32, z: f32) -> Self {
            Self { x, y, z }
        }
        pub fn add(self, o: JVec3) -> Self {
            Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
        }
        pub fn add3(self, x: f32, y: f32, z: f32) -> Self {
            Self::new(self.x + x, self.y + y, self.z + z)
        }
        pub fn sub(self, o: JVec3) -> Self {
            Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
        }
        pub fn mul_s(self, s: f32) -> Self {
            Self::new(self.x * s, self.y * s, self.z * s)
        }
        pub fn mul_v(self, o: JVec3) -> Self {
            Self::new(self.x * o.x, self.y * o.y, self.z * o.z)
        }
        pub fn negate(self) -> Self {
            Self::new(-self.x, -self.y, -self.z)
        }
        pub fn dot(self, o: JVec3) -> f32 {
            self.x * o.x + self.y * o.y + self.z * o.z
        }
        pub fn length(self) -> f32 {
            self.dot(self).sqrt()
        }
        pub fn length_squared(self) -> f32 {
            self.dot(self)
        }
        /// JOML's normalize: divides by the length (a zero vector becomes NaNs, as in Java).
        pub fn normalize(self) -> Self {
            let inv = 1.0 / self.length();
            self.mul_s(inv)
        }
        pub fn cross(self, o: JVec3) -> Self {
            Self::new(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)
        }
        /// JOML's lerp: this + (other - this) * t.
        pub fn lerp(self, o: JVec3, t: f32) -> Self {
            Self::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t, self.z + (o.z - self.z) * t)
        }
        pub fn distance(self, o: JVec3) -> f32 {
            self.sub(o).length()
        }
    }

    /// JOML's Quaternionf (x, y, z, w).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct JQuat {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub w: f32,
    }
    impl Default for JQuat {
        fn default() -> Self {
            Self::IDENTITY
        }
    }
    impl JQuat {
        pub const IDENTITY: JQuat = JQuat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
        pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
            Self { x, y, z, w }
        }
        /// `rotationX(angle)`: a turn about +X by `angle` radians (Axis.XP.rotation).
        pub fn rotation_x(a: f32) -> Self {
            let (s, c) = (a * 0.5).sin_cos();
            Self::new(s, 0.0, 0.0, c)
        }
        pub fn rotation_y(a: f32) -> Self {
            let (s, c) = (a * 0.5).sin_cos();
            Self::new(0.0, s, 0.0, c)
        }
        pub fn rotation_z(a: f32) -> Self {
            let (s, c) = (a * 0.5).sin_cos();
            Self::new(0.0, 0.0, s, c)
        }
        pub fn normalize(self) -> Self {
            let inv = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
            Self::new(self.x * inv, self.y * inv, self.z * inv, self.w * inv)
        }
        /// this * q.
        pub fn mul(self, q: JQuat) -> Self {
            Self::new(
                self.w * q.x + self.x * q.w + self.y * q.z - self.z * q.y,
                self.w * q.y - self.x * q.z + self.y * q.w + self.z * q.x,
                self.w * q.z + self.x * q.y - self.y * q.x + self.z * q.w,
                self.w * q.w - self.x * q.x - self.y * q.y - self.z * q.z,
            )
        }
        pub fn rotate_x(self, a: f32) -> Self {
            self.mul(Self::rotation_x(a))
        }
        pub fn rotate_y(self, a: f32) -> Self {
            self.mul(Self::rotation_y(a))
        }
        pub fn rotate_z(self, a: f32) -> Self {
            self.mul(Self::rotation_z(a))
        }
    }

    /// JOML's Matrix3f, column-major, from three column vectors.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct JMat3 {
        pub m: [f32; 9],
    }
    impl JMat3 {
        pub fn from_cols(c0: JVec3, c1: JVec3, c2: JVec3) -> Self {
            Self { m: [c0.x, c0.y, c0.z, c1.x, c1.y, c1.z, c2.x, c2.y, c2.z] }
        }
    }

    /// JOML's Matrix4f: column-major, `m[col * 4 + row]`; `a.mul(b)` is a * b, and translate/rotate/scale
    /// multiply on the right, as JOML does.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct JMat4 {
        pub m: [f32; 16],
    }
    impl Default for JMat4 {
        fn default() -> Self {
            Self::IDENTITY
        }
    }
    impl JMat4 {
        pub const IDENTITY: JMat4 = JMat4 { m: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] };
        pub fn identity() -> Self {
            Self::IDENTITY
        }
        fn at(&self, row: usize, col: usize) -> f32 {
            self.m[col * 4 + row]
        }
        pub fn mul(self, r: JMat4) -> Self {
            let mut out = [0.0f32; 16];
            for col in 0..4 {
                for row in 0..4 {
                    let mut v = 0.0;
                    for k in 0..4 {
                        v += self.at(row, k) * r.at(k, col);
                    }
                    out[col * 4 + row] = v;
                }
            }
            Self { m: out }
        }
        pub fn translation(x: f32, y: f32, z: f32) -> Self {
            let mut t = Self::IDENTITY;
            t.m[12] = x;
            t.m[13] = y;
            t.m[14] = z;
            t
        }
        pub fn translate(self, x: f32, y: f32, z: f32) -> Self {
            self.mul(Self::translation(x, y, z))
        }
        pub fn translate_v(self, v: JVec3) -> Self {
            self.translate(v.x, v.y, v.z)
        }
        pub fn scaling(x: f32, y: f32, z: f32) -> Self {
            let mut t = Self::IDENTITY;
            t.m[0] = x;
            t.m[5] = y;
            t.m[10] = z;
            t
        }
        pub fn scale(self, x: f32, y: f32, z: f32) -> Self {
            self.mul(Self::scaling(x, y, z))
        }
        pub fn scale_s(self, s: f32) -> Self {
            self.scale(s, s, s)
        }
        /// The rotation a unit quaternion stands for.
        pub fn rotation(q: JQuat) -> Self {
            let (x, y, z, w) = (q.x, q.y, q.z, q.w);
            let (xx, yy, zz) = (x * x, y * y, z * z);
            let (xy, xz, yz, wx, wy, wz) = (x * y, x * z, y * z, w * x, w * y, w * z);
            Self {
                m: [
                    1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0,
                    2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx), 0.0,
                    2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy), 0.0,
                    0.0, 0.0, 0.0, 1.0,
                ],
            }
        }
        pub fn rotate(self, q: JQuat) -> Self {
            self.mul(Self::rotation(q))
        }
        pub fn rotate_x(self, a: f32) -> Self {
            self.rotate(JQuat::rotation_x(a))
        }
        pub fn rotate_y(self, a: f32) -> Self {
            self.rotate(JQuat::rotation_y(a))
        }
        pub fn rotate_z(self, a: f32) -> Self {
            self.rotate(JQuat::rotation_z(a))
        }
        /// JOML's rotateXYZ: about X, then Y, then Z.
        pub fn rotate_xyz(self, x: f32, y: f32, z: f32) -> Self {
            self.rotate_x(x).rotate_y(y).rotate_z(z)
        }
        /// JOML's rotateAround(q, ox, oy, oz): about the point (ox, oy, oz).
        pub fn rotate_around(self, q: JQuat, ox: f32, oy: f32, oz: f32) -> Self {
            self.translate(ox, oy, oz).rotate(q).translate(-ox, -oy, -oz)
        }
        /// The general inverse (JOML's invert); a singular matrix gives infinities, as in Java.
        pub fn invert(self) -> Self {
            let a = &self.m;
            let mut inv = [0.0f32; 16];
            inv[0] = a[5] * a[10] * a[15] - a[5] * a[11] * a[14] - a[9] * a[6] * a[15] + a[9] * a[7] * a[14] + a[13] * a[6] * a[11] - a[13] * a[7] * a[10];
            inv[4] = -a[4] * a[10] * a[15] + a[4] * a[11] * a[14] + a[8] * a[6] * a[15] - a[8] * a[7] * a[14] - a[12] * a[6] * a[11] + a[12] * a[7] * a[10];
            inv[8] = a[4] * a[9] * a[15] - a[4] * a[11] * a[13] - a[8] * a[5] * a[15] + a[8] * a[7] * a[13] + a[12] * a[5] * a[11] - a[12] * a[7] * a[9];
            inv[12] = -a[4] * a[9] * a[14] + a[4] * a[10] * a[13] + a[8] * a[5] * a[14] - a[8] * a[6] * a[13] - a[12] * a[5] * a[10] + a[12] * a[6] * a[9];
            inv[1] = -a[1] * a[10] * a[15] + a[1] * a[11] * a[14] + a[9] * a[2] * a[15] - a[9] * a[3] * a[14] - a[13] * a[2] * a[11] + a[13] * a[3] * a[10];
            inv[5] = a[0] * a[10] * a[15] - a[0] * a[11] * a[14] - a[8] * a[2] * a[15] + a[8] * a[3] * a[14] + a[12] * a[2] * a[11] - a[12] * a[3] * a[10];
            inv[9] = -a[0] * a[9] * a[15] + a[0] * a[11] * a[13] + a[8] * a[1] * a[15] - a[8] * a[3] * a[13] - a[12] * a[1] * a[11] + a[12] * a[3] * a[9];
            inv[13] = a[0] * a[9] * a[14] - a[0] * a[10] * a[13] - a[8] * a[1] * a[14] + a[8] * a[2] * a[13] + a[12] * a[1] * a[10] - a[12] * a[2] * a[9];
            inv[2] = a[1] * a[6] * a[15] - a[1] * a[7] * a[14] - a[5] * a[2] * a[15] + a[5] * a[3] * a[14] + a[13] * a[2] * a[7] - a[13] * a[3] * a[6];
            inv[6] = -a[0] * a[6] * a[15] + a[0] * a[7] * a[14] + a[4] * a[2] * a[15] - a[4] * a[3] * a[14] - a[12] * a[2] * a[7] + a[12] * a[3] * a[6];
            inv[10] = a[0] * a[5] * a[15] - a[0] * a[7] * a[13] - a[4] * a[1] * a[15] + a[4] * a[3] * a[13] + a[12] * a[1] * a[7] - a[12] * a[3] * a[5];
            inv[14] = -a[0] * a[5] * a[14] + a[0] * a[6] * a[13] + a[4] * a[1] * a[14] - a[4] * a[2] * a[13] - a[12] * a[1] * a[6] + a[12] * a[2] * a[5];
            inv[3] = -a[1] * a[6] * a[11] + a[1] * a[7] * a[10] + a[5] * a[2] * a[11] - a[5] * a[3] * a[10] - a[9] * a[2] * a[7] + a[9] * a[3] * a[6];
            inv[7] = a[0] * a[6] * a[11] - a[0] * a[7] * a[10] - a[4] * a[2] * a[11] + a[4] * a[3] * a[10] + a[8] * a[2] * a[7] - a[8] * a[3] * a[6];
            inv[11] = -a[0] * a[5] * a[11] + a[0] * a[7] * a[9] + a[4] * a[1] * a[11] - a[4] * a[3] * a[9] - a[8] * a[1] * a[7] + a[8] * a[3] * a[5];
            inv[15] = a[0] * a[5] * a[10] - a[0] * a[6] * a[9] - a[4] * a[1] * a[10] + a[4] * a[2] * a[9] + a[8] * a[1] * a[6] - a[8] * a[2] * a[5];
            let det = a[0] * inv[0] + a[1] * inv[4] + a[2] * inv[8] + a[3] * inv[12];
            let d = 1.0 / det;
            for v in inv.iter_mut() {
                *v *= d;
            }
            Self { m: inv }
        }
        /// The point v moved by this matrix (w = 1).
        pub fn transform_position(&self, v: JVec3) -> JVec3 {
            JVec3::new(
                self.at(0, 0) * v.x + self.at(0, 1) * v.y + self.at(0, 2) * v.z + self.at(0, 3),
                self.at(1, 0) * v.x + self.at(1, 1) * v.y + self.at(1, 2) * v.z + self.at(1, 3),
                self.at(2, 0) * v.x + self.at(2, 1) * v.y + self.at(2, 2) * v.z + self.at(2, 3),
            )
        }
        /// The direction v turned by this matrix (w = 0).
        pub fn transform_direction(&self, v: JVec3) -> JVec3 {
            JVec3::new(
                self.at(0, 0) * v.x + self.at(0, 1) * v.y + self.at(0, 2) * v.z,
                self.at(1, 0) * v.x + self.at(1, 1) * v.y + self.at(1, 2) * v.z,
                self.at(2, 0) * v.x + self.at(2, 1) * v.y + self.at(2, 2) * v.z,
            )
        }
        /// JOML's set3x3: the upper-left 3x3 from `m`, the rest kept.
        pub fn set3x3(self, m: JMat3) -> Self {
            let mut out = self;
            for col in 0..3 {
                for row in 0..3 {
                    out.m[col * 4 + row] = m.m[col * 3 + row];
                }
            }
            out
        }
        /// JOML's transpose3x3: the upper-left 3x3 transposed, the rest kept.
        pub fn transpose3x3(self) -> Self {
            let mut out = self;
            for col in 0..3 {
                for row in 0..3 {
                    out.m[col * 4 + row] = self.m[row * 4 + col];
                }
            }
            out
        }
        pub fn get_translation(&self) -> JVec3 {
            JVec3::new(self.m[12], self.m[13], self.m[14])
        }
    }

    /// Minecraft's PoseStack: a stack of matrices (normals aren't kept: Horizon's hand renderer takes one matrix).
    #[derive(Clone, Debug)]
    pub struct JPoseStack {
        pub stack: Vec<JMat4>,
    }
    impl Default for JPoseStack {
        fn default() -> Self {
            Self { stack: vec![JMat4::IDENTITY] }
        }
    }
    impl JPoseStack {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn of(m: JMat4) -> Self {
            Self { stack: vec![m] }
        }
        fn top(&mut self) -> &mut JMat4 {
            self.stack.last_mut().expect("PoseStack is empty")
        }
        pub fn push_pose(&mut self) {
            let m = *self.top();
            self.stack.push(m);
        }
        pub fn pop_pose(&mut self) {
            self.stack.pop();
        }
        /// Writes the top matrix (Java changed `pose.last().pose()` in place).
        pub fn set_last(&mut self, m: JMat4) {
            *self.top() = m;
        }
        pub fn last_pose(&self) -> JMat4 {
            *self.stack.last().expect("PoseStack is empty")
        }
        pub fn set_identity(&mut self) {
            *self.top() = JMat4::IDENTITY;
        }
        pub fn translate(&mut self, x: f32, y: f32, z: f32) {
            let m = self.top();
            *m = m.translate(x, y, z);
        }
        pub fn scale(&mut self, x: f32, y: f32, z: f32) {
            let m = self.top();
            *m = m.scale(x, y, z);
        }
        pub fn mul_pose(&mut self, q: JQuat) {
            let m = self.top();
            *m = m.rotate(q);
        }
        pub fn mul_pose_matrix(&mut self, r: JMat4) {
            let m = self.top();
            *m = m.mul(r);
        }
        pub fn rotate_around(&mut self, q: JQuat, x: f32, y: f32, z: f32) {
            let m = self.top();
            *m = m.rotate_around(q, x, y, z);
        }
    }

    #[cfg(test)]
    mod joml_tests {
        use super::*;

        fn close(a: JVec3, b: JVec3) -> bool {
            (a.x - b.x).abs() < 1e-5 && (a.y - b.y).abs() < 1e-5 && (a.z - b.z).abs() < 1e-5
        }

        #[test]
        fn rotations_turn_axes_as_joml_does() {
            let q = JQuat::rotation_x(std::f32::consts::FRAC_PI_2);
            // A positive turn about X takes +Y to +Z (right-handed), as JOML's rotationX.
            assert!(close(JMat4::rotation(q).transform_position(JVec3::new(0.0, 1.0, 0.0)), JVec3::new(0.0, 0.0, 1.0)));
            let q = JQuat::rotation_y(std::f32::consts::FRAC_PI_2);
            assert!(close(JMat4::rotation(q).transform_position(JVec3::new(0.0, 0.0, 1.0)), JVec3::new(1.0, 0.0, 0.0)));
            let q = JQuat::rotation_z(std::f32::consts::FRAC_PI_2);
            assert!(close(JMat4::rotation(q).transform_position(JVec3::new(1.0, 0.0, 0.0)), JVec3::new(0.0, 1.0, 0.0)));
        }

        #[test]
        fn translate_then_rotate_multiplies_on_the_right() {
            // JOML: new Matrix4f().translate(1, 0, 0).rotateZ(90) moves (1, 0, 0) to (1, 1, 0).
            let m = JMat4::identity().translate(1.0, 0.0, 0.0).rotate_z(std::f32::consts::FRAC_PI_2);
            assert!(close(m.transform_position(JVec3::new(1.0, 0.0, 0.0)), JVec3::new(1.0, 1.0, 0.0)));
        }

        #[test]
        fn invert_undoes() {
            let m = JMat4::identity().translate(1.0, 2.0, 3.0).rotate_x(0.7).rotate_y(-0.3).scale(2.0, 0.5, 1.5);
            let p = JVec3::new(0.3, -1.2, 4.0);
            assert!(close(m.invert().transform_position(m.transform_position(p)), p));
            assert!(close(m.mul(m.invert()).transform_position(p), p));
        }

        #[test]
        fn rotate_around_keeps_its_point() {
            let m = JMat4::identity().rotate_around(JQuat::rotation_y(1.1), 0.5, 0.25, 0.0);
            assert!(close(m.transform_position(JVec3::new(0.5, 0.25, 0.0)), JVec3::new(0.5, 0.25, 0.0)));
        }

        #[test]
        fn pose_stack_push_pop() {
            let mut p = JPoseStack::new();
            p.translate(1.0, 0.0, 0.0);
            p.push_pose();
            p.scale(2.0, 2.0, 2.0);
            assert!(close(p.last_pose().transform_position(JVec3::new(1.0, 0.0, 0.0)), JVec3::new(3.0, 0.0, 0.0)));
            p.pop_pose();
            assert!(close(p.last_pose().transform_position(JVec3::new(1.0, 0.0, 0.0)), JVec3::new(2.0, 0.0, 0.0)));
        }

        #[test]
        fn quaternion_matrix_agrees_with_axis_turns() {
            let q = JQuat::rotation_x(0.4).mul(JQuat::rotation_y(-0.9));
            let a = JMat4::rotation(q);
            let b = JMat4::identity().rotate_x(0.4).rotate_y(-0.9);
            let p = JVec3::new(1.0, 2.0, 3.0);
            assert!(close(a.transform_position(p), b.transform_position(p)));
        }

        #[test]
        fn set3x3_and_transpose() {
            let m3 = JMat3::from_cols(JVec3::new(0.0, 1.0, 0.0), JVec3::new(-1.0, 0.0, 0.0), JVec3::new(0.0, 0.0, 1.0));
            let m = JMat4::identity().set3x3(m3);
            assert!(close(m.transform_position(JVec3::new(1.0, 0.0, 0.0)), JVec3::new(0.0, 1.0, 0.0)));
            assert!(close(m.transpose3x3().transform_position(JVec3::new(0.0, 1.0, 0.0)), JVec3::new(1.0, 0.0, 0.0)));
        }
    }

    /// FabricLoader.getModContainer: this mod (its fabric.mod.json) and Minecraft (the version Horizon plays).
    pub fn j_mod(id: &str) -> Option<JMod> {
        match id {
            "vulkanmod" => Some(JMod::new("vulkanmod", "0.6.8+26.3", &[("depends", "fabricloader", &[">=0.18.6"]), ("depends", "minecraft", &[">=26.1"])])),
            "minecraft" => Some(JMod::new("minecraft", "26.3", &[])),
            _ => None,
        }
    }


    /// Fabric Loader's `Version` and `SemanticVersion` (net.fabricmc.loader.api), parsed and compared as the loader
    /// does: numbers separated by dots, then an optional `-pre.release` and `+build` (which comparing ignores).
    #[derive(Clone, Debug, PartialEq)]
    pub struct JVersion {
        friendly: String,
        /// The numbers of a semantic version; None for one that isn't (compared as text).
        parts: Option<Vec<i64>>,
        pre: Option<String>,
    }

    impl JVersion {
        /// `SemanticVersion.parse`: an error (VersionParsingException) when it isn't one.
        pub fn parse_semantic(s: &str) -> std::io::Result<JVersion> {
            Self::semantic(s).ok_or_else(|| std::io::Error::other(format!("Could not parse version number '{s}'")))
        }

        /// `Version.parse`: a semantic version when it is one, else the text itself.
        pub fn parse(s: &str) -> std::io::Result<JVersion> {
            if s.is_empty() {
                return Err(std::io::Error::other("Empty version"));
            }
            Ok(Self::semantic(s).unwrap_or(JVersion { friendly: s.to_string(), parts: None, pre: None }))
        }

        fn semantic(s: &str) -> Option<JVersion> {
            let core_pre = s.split_once('+').map_or(s, |(c, _)| c);
            let (core, pre) = match core_pre.split_once('-') {
                Some((c, p)) => (c, Some(p.to_string())),
                None => (core_pre, None),
            };
            if let Some(p) = &pre {
                if !p.split('.').all(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')) {
                    return None;
                }
            }
            let mut parts = Vec::new();
            for c in core.split('.') {
                if c.is_empty() || !c.chars().all(|ch| ch.is_ascii_digit()) {
                    return None;
                }
                parts.push(c.parse::<i64>().ok()?);
            }
            Some(JVersion { friendly: s.to_string(), parts: Some(parts), pre })
        }

        /// `getFriendlyString`: the text it was parsed from.
        pub fn friendly_string(&self) -> String {
            self.friendly.clone()
        }

        /// `compareTo`: below, at or above zero.
        pub fn compare_to(&self, o: &JVersion) -> i32 {
            let (Some(a), Some(b)) = (&self.parts, &o.parts) else {
                return match self.friendly.cmp(&o.friendly) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
            };
            for i in 0..a.len().max(b.len()) {
                let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
                if x != y {
                    return if x < y { -1 } else { 1 };
                }
            }
            match (&self.pre, &o.pre) {
                (None, None) => 0,
                (None, Some(_)) => 1,
                (Some(_), None) => -1,
                (Some(p), Some(q)) => {
                    let (ps, qs): (Vec<&str>, Vec<&str>) = (p.split('.').collect(), q.split('.').collect());
                    for i in 0..ps.len().min(qs.len()) {
                        let c = match (ps[i].parse::<i64>(), qs[i].parse::<i64>()) {
                            (Ok(x), Ok(y)) => x.cmp(&y),
                            (Ok(_), Err(_)) => std::cmp::Ordering::Less,
                            (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
                            _ => ps[i].cmp(qs[i]),
                        };
                        if c != std::cmp::Ordering::Equal {
                            return if c == std::cmp::Ordering::Less { -1 } else { 1 };
                        }
                    }
                    (ps.len() as i32 - qs.len() as i32).signum()
                }
            }
        }

        /// The numbers, a missing one as 0.
        fn part(&self, i: usize) -> i64 {
            self.parts.as_ref().and_then(|p| p.get(i).copied()).unwrap_or(0)
        }
    }

    /// One dependency of a mod's fabric.mod.json: its kind (`depends`, `recommends`, `suggests`, `conflicts`,
    /// `breaks`), the mod it names and its version ranges (any one matches).
    #[derive(Clone, Debug, PartialEq)]
    pub struct JModDep {
        pub kind: &'static str,
        pub mod_id: String,
        pub ranges: Vec<String>,
    }

    impl JModDep {
        /// `ModDependency.matches`: a range's space-separated predicates all hold (`>=1.2`, `~26.2`, `^1.0`, `1.20.x`, `*`).
        pub fn matches(&self, v: &JVersion) -> bool {
            self.ranges.iter().any(|r| r.split_whitespace().all(|p| Self::predicate(p, v)))
        }

        fn predicate(p: &str, v: &JVersion) -> bool {
            if p == "*" {
                return true;
            }
            for op in [">=", "<=", ">", "<", "=", "~", "^"] {
                if let Some(rest) = p.strip_prefix(op) {
                    let Ok(w) = JVersion::parse(rest) else {
                        return false;
                    };
                    let c = v.compare_to(&w);
                    return match op {
                        ">=" => c >= 0,
                        "<=" => c <= 0,
                        ">" => c > 0,
                        "<" => c < 0,
                        "=" => c == 0,
                        // ~1.2.3: at least it, below the next minor version.
                        "~" => c >= 0 && v.part(0) == w.part(0) && v.part(1) == w.part(1),
                        // ^1.2.3: at least it, below the next major version.
                        _ => c >= 0 && v.part(0) == w.part(0),
                    };
                }
            }
            // 1.20.x: the numbers before the x match.
            if p.ends_with(".x") || p.ends_with(".X") || p.ends_with(".*") {
                let head: Vec<&str> = p[..p.len() - 2].split('.').collect();
                return head.iter().enumerate().all(|(i, h)| h.parse::<i64>().is_ok_and(|n| v.part(i) == n));
            }
            JVersion::parse(p).is_ok_and(|w| v.compare_to(&w) == 0)
        }
    }

    /// A mod as `FabricLoader.getModContainer` gives it, from the mod's fabric.mod.json at conversion time
    /// (and `minecraft` as the version Horizon plays).
    #[derive(Clone, Debug, PartialEq)]
    pub struct JMod {
        pub id: String,
        pub version: JVersion,
        pub deps: Vec<JModDep>,
    }

    impl JMod {
        pub fn new(id: &str, version: &str, deps: &[(&'static str, &str, &[&str])]) -> JMod {
            JMod {
                id: id.to_string(),
                version: JVersion::parse(version).unwrap_or(JVersion { friendly: version.to_string(), parts: None, pre: None }),
                deps: deps
                    .iter()
                    .map(|(k, m, r)| JModDep { kind: k, mod_id: m.to_string(), ranges: r.iter().map(|s| s.to_string()).collect() })
                    .collect(),
            }
        }
    }

    #[cfg(test)]
    mod jfabric_tests {
        use super::*;

        #[test]
        fn versions_compare_like_fabric() {
            let v = |s| JVersion::parse_semantic(s).unwrap();
            assert!(v("1.0.1").compare_to(&v("1.0.0")) > 0);
            assert!(v("1.0").compare_to(&v("1.0.0")) == 0);
            assert!(v("1.0.0-beta.2").compare_to(&v("1.0.0")) < 0);
            assert!(v("1.0.0-beta.10").compare_to(&v("1.0.0-beta.2")) > 0);
            assert!(v("1.0.0+26.2").compare_to(&v("1.0.0+26.3")) == 0);
            assert!(JVersion::parse_semantic("v1.0").is_err());
            assert_eq!(v("1.2.3+build").friendly_string(), "1.2.3+build");
        }

        #[test]
        fn dependency_ranges() {
            let m = JMod::new("x", "1.0.0", &[("depends", "minecraft", &["~26.2"]), ("depends", "fabricloader", &[">=0.19"])]);
            let mc = |s| JVersion::parse(s).unwrap();
            assert!(m.deps[0].matches(&mc("26.2")));
            assert!(m.deps[0].matches(&mc("26.2.1")));
            assert!(!m.deps[0].matches(&mc("26.3")));
            assert!(m.deps[1].matches(&mc("0.19.2")));
            let d = JModDep { kind: "depends", mod_id: "minecraft".into(), ranges: vec!["1.20.x".into(), ">=26.3 <27".into()] };
            assert!(d.matches(&mc("1.20.4")) && d.matches(&mc("26.3")) && !d.matches(&mc("27.0")));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_set() {
        assert_eq!(ID, "vulkanmod");
        assert!(!NAME.is_empty());
        assert!(KEYS.iter().all(|[n, _, d]| !n.is_empty() && d.starts_with("key.")));
    }
}
