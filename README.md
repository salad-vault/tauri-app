# SaladVault - Application Desktop / Desktop Application

> Gestionnaire de mots de passe **Zero-Knowledge** a double verrouillage (Dual-Lock). Client desktop construit avec **Tauri 2** + **Leptos 0.8** (CSR/WASM).

> **Zero-Knowledge** password manager with Dual-Lock protection. Desktop client built with **Tauri 2** + **Leptos 0.8** (CSR/WASM).

---

## Table of contents / Table des matieres

- [Francais](#-francais)
- [English](#-english)

---

## :fr: Francais

### Table des matieres

- [Architecture](#architecture)
- [Stack technique](#stack-technique)
- [Structure du projet](#structure-du-projet)
- [Modele cryptographique](#modele-cryptographique)
- [Vocabulaire metier](#vocabulaire-metier)
- [Commandes Tauri](#commandes-tauri)
- [Base de donnees](#base-de-donnees)
- [Composants frontend](#composants-frontend)
- [Gestion de l'etat](#gestion-de-letat)
- [Fonctionnalites de securite](#fonctionnalites-de-securite)
- [Configuration](#configuration)
- [Developpement](#developpement)
- [Build de production](#build-de-production)

---

### Architecture

```
+--------------------------------------------------+
|                    Tauri Webview                  |
|  +--------------------------------------------+  |
|  |         Frontend (Leptos 0.8 CSR)          |  |
|  |         Compile en WASM via Trunk          |  |
|  |                                            |  |
|  |  App --> Login / Register / Dashboard       |  |
|  |         Settings / SaladierView            |  |
|  |         PanicUnlock / Recovery             |  |
|  +-------------------+------------------------+  |
|                      | invoke("command", args)    |
|  +-------------------v------------------------+  |
|  |          Backend (Tauri 2 + Rust)          |  |
|  |                                            |  |
|  |  Commands --> Crypto --> Database (SQLite)   |  |
|  |              |                             |  |
|  |              +-- XChaCha20-Poly1305        |  |
|  |              +-- Argon2id (OWASP)          |  |
|  |              +-- HKDF-SHA256               |  |
|  |              +-- HMAC-SHA256               |  |
|  +--------------------------------------------+  |
+--------------------------------------------------+
         |                          |
    device_secret.key          saladvault.db
    (32 bytes, local)        (blobs chiffres)
```

Le frontend communique avec le backend exclusivement via `wasm_bindgen` et l'API `invoke()` de Tauri. Toutes les operations cryptographiques sont effectuees cote backend. La base de donnees ne contient que des blobs chiffres.

---

### Stack technique

| Couche | Technologie | Version |
|--------|-------------|---------|
| Langage | Rust | Edition 2021 |
| Framework desktop | Tauri | 2.x |
| Framework UI | Leptos (CSR) | 0.8 |
| Compilation WASM | Trunk | - |
| BDD locale | SQLite (WAL) | via `rusqlite` bundled |
| Chiffrement symetrique | XChaCha20-Poly1305 | `chacha20poly1305` 0.10 |
| KDF | Argon2id | `argon2` 0.5 |
| Derivation de cles | HKDF-SHA256 | `hkdf` 0.12 |
| Indexation aveugle | HMAC-SHA256 | `hmac` 0.12 + `sha2` 0.10 |
| Phrase de recuperation | BIP39 | `bip39` 2.x |
| QR Code | qrcodegen | 1.8 |
| Import/Export | CSV + XML | `csv` 1 + `quick-xml` 0.36 |

---

### Structure du projet

```
rust-app/
+-- Cargo.toml                  # Manifest frontend + declaration workspace
+-- index.html                  # Point d'entree HTML (Trunk)
+-- styles.css                  # Feuille de styles (theme sombre, BEM-like)
+-- public/                     # Assets statiques
|
+-- src/                        # Frontend Leptos (CSR/WASM)
|   +-- main.rs                 # Bootstrap : panic hook + mount_to_body
|   +-- app.rs                  # Composant racine, navigation, auto-lock
|   +-- components/
|       +-- mod.rs              # Re-exports des modules
|       +-- login.rs            # Ecran de connexion
|       +-- register.rs         # Creation de compte
|       +-- dashboard.rs        # Hub principal (liste des Saladiers)
|       +-- nag_screen.rs       # Avertissement sauvegarde Kit de Secours
|       +-- recovery.rs         # Phrase de recuperation BIP39
|       +-- panic_unlock.rs     # Deverrouillage Saladier (Panic Mode)
|       +-- saladier_view.rs    # Vue des Feuilles + copie presse-papiers
|       +-- feuille_form.rs     # Formulaire creation/edition d'entree
|       +-- settings.rs         # Conteneur Parametres + sous-navigation
|       +-- settings_security.rs    # Verrouillage, presse-papiers, screenshots
|       +-- settings_keys.rs        # Gestion cles, Kit de Secours, zone danger
|       +-- settings_devices.rs     # Appareils, QR Code
|       +-- settings_saladiers.rs   # Parametres des Saladiers
|       +-- settings_data.rs        # Import/Export
|       +-- settings_privacy.rs     # Vie privee, rapports de crash
|       +-- settings_general.rs     # Theme, generateur, Dead Man's Switch
|       +-- settings_subscription.rs # Abonnement (Freemium)
|       +-- password_utils.rs       # Validation force mot de passe
|
+-- src-tauri/                  # Backend Tauri (natif)
    +-- Cargo.toml              # Manifest backend
    +-- tauri.conf.json         # Configuration Tauri
    +-- capabilities/
    |   +-- default.json        # Permissions plugins (dialog, clipboard)
    +-- icons/                  # Icones application
    +-- src/
        +-- main.rs             # Point d'entree -> lib::run()
        +-- lib.rs              # Setup Tauri, plugins, enregistrement commandes
        +-- error.rs            # Enum AppError (messages generiques FR)
        +-- state.rs            # AppState : session, cache cles, DB
        +-- commands/
        |   +-- mod.rs
        |   +-- auth.rs         # register, unlock, lock, verify, change pwd
        |   +-- device.rs       # init/check/move/export/regenerate device key
        |   +-- saladiers.rs    # CRUD Saladiers + tentatives echouees
        |   +-- feuilles.rs     # CRUD Feuilles (entrees chiffrees)
        |   +-- recovery.rs     # Phrase BIP39, restauration
        |   +-- settings.rs     # Parametres, clipboard, activite, screenshots
        |   +-- password_gen.rs # Generateur de mots de passe
        |   +-- import_export.rs # Import CSV/XML, export JSON/CSV
        |   +-- maintenance.rs  # VACUUM, integrity check
        +-- crypto/
        |   +-- mod.rs
        |   +-- keys.rs         # Generation/chargement cles, reconstruction MasterKey
        |   +-- xchacha.rs      # XChaCha20-Poly1305 encrypt/decrypt
        |   +-- argon2_kdf.rs   # Argon2id (params OWASP)
        |   +-- blind_index.rs  # HMAC-SHA256 pour lookup sans fuite d'email
        +-- db/
        |   +-- mod.rs          # open_database (SQLite WAL)
        |   +-- schema.rs       # CREATE TABLE (users, saladiers, feuilles, settings)
        |   +-- users.rs        # CRUD utilisateurs
        |   +-- saladiers.rs    # CRUD Saladiers + compteur tentatives
        |   +-- feuilles.rs     # CRUD Feuilles
        |   +-- settings.rs     # get/save settings (JSON)
        +-- models/
            +-- mod.rs
            +-- user.rs         # Struct User
            +-- saladier.rs     # Structs Saladier, SaladierInfo
            +-- feuille.rs      # Structs Feuille, FeuilleData, FeuilleInfo
            +-- settings.rs     # Struct UserSettings + enums
```

---

### Modele cryptographique

#### Principe Zero-Knowledge

Aucune donnee en clair n'est jamais stockee. La base de donnees SQLite ne contient que des blobs chiffres. Le serveur (scope futur) ne verra jamais de mot de passe, d'email ou d'entree en clair.

#### Dual-Lock : Reconstruction de la Master Key

La cle maitre necessite **deux facteurs** pour etre reconstruite :

1. **Mot de passe maitre** (memorise par l'utilisateur)
2. **Cle de peripherique** (`device_secret.key`, 32 octets, stockee sur l'appareil)

```
Etape 1 : derived_key = Argon2id(password, salt_master)
              Params OWASP : m=64MB, t=3, p=4, output=32 bytes

Etape 2 : prk = HKDF-Extract(salt=device_key, ikm=derived_key)

Etape 3 : master_key = HKDF-Expand(prk, info="SaladVault_MasterKey_v2", 32 bytes)

Etape 4 : Zeroize(derived_key)  // materiel intermediaire efface
```

#### Chiffrement des donnees

| Donnee | Cle utilisee | Algorithme |
|--------|-------------|------------|
| Noms de Saladiers | Master Key | XChaCha20-Poly1305 (nonce 24 octets) |
| Token de verification compte | Master Key | XChaCha20-Poly1305 |
| Token de verification Saladier | K_S (cle Saladier) | XChaCha20-Poly1305 |
| Feuilles (entrees) | K_S (cle Saladier) | XChaCha20-Poly1305 |

#### Derivation de K_S (cle de Saladier)

Chaque Saladier possede sa propre cle derivee independamment :

```
K_S = Argon2id(saladier_password, salt_saladier)
    Params : m=64MB, t=3, p=4, output=32 bytes
```

#### Indexation aveugle (Blind Index)

L'email de l'utilisateur n'est jamais stocke. Un identifiant deterministe est calcule via :

```
user_id = HMAC-SHA256(
    key  = "SaladVault_BlindIndex_Pepper_v1_CHANGE_IN_PRODUCTION",
    msg  = normalize(email) + static_salt
)
```

Ou `normalize(email) = email.trim().to_lowercase()`.

#### Zeroisage memoire

Tout materiel cryptographique est protege par `zeroize` :

- `MasterKey` : implemente `Zeroize` + `Drop` (32 octets effaces a la destruction)
- `Session` : zeroise `master_key_bytes` et `user_id` au `Drop`
- Cache `saladier_keys` : chaque cle K_S zeroisee individuellement au logout

---

### Vocabulaire metier

| Terme | Signification | Description |
|-------|--------------|-------------|
| **Potager** | Compte utilisateur | L'espace global de l'utilisateur |
| **Saladier** | Vault / Conteneur | Base de donnees isolee, chiffree independamment |
| **Feuille** | Entree | Un couple identifiant/mot de passe (+ URL, notes) |
| **Ingredient Secret** | Cle locale | Fichier `device_secret.key` (32 octets) |
| **Kit de Secours** | Phrase de recuperation | 12 mots BIP39 pour restaurer la cle locale |
| **Panic Mode** | Double verrouillage | Chaque Saladier a son propre mot de passe |

---

### Commandes Tauri

#### Authentification (`commands/auth.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `register` | `email`, `masterPassword` | Cree un compte, genere la device key, etablit la session |
| `unlock` | `email`, `masterPassword` | Reconstruit la Master Key, verifie, etablit la session |
| `lock` | - | Efface la session et le cache des cles Saladier |
| `is_unlocked` | - | Retourne `true` si une session est active |
| `verify_master_password` | `masterPassword` | Verifie le mot de passe sans modifier la session |
| `change_master_password` | `currentPassword`, `newPassword` | Re-chiffre `k_cloud_enc` avec la nouvelle cle |

#### Cle de peripherique (`commands/device.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `init_device_key` | - | Genere `device_secret.key` si absent |
| `check_device_key` | - | Verifie l'existence du fichier cle |
| `get_device_key_path` | - | Retourne le chemin absolu de la cle |
| `move_device_key` | `newPath` | Deplace la cle (ex: vers une cle USB) |
| `export_device_key_qrcode` | - | Exporte la cle en base64 |
| `generate_device_key_qr_svg` | - | Genere un QR code SVG de la cle |
| `regenerate_device_key` | `masterPassword` | Regenere la cle, re-chiffre toutes les donnees |

#### Saladiers (`commands/saladiers.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `create_saladier` | `name`, `password`, `hidden` | Cree un Saladier avec sa propre cle K_S |
| `list_saladiers` | - | Liste les Saladiers visibles (noms dechiffres) |
| `open_saladier` | `uuid`, `password` | Deverrouille un Saladier, met K_S en cache |
| `delete_saladier` | `uuid`, `masterPassword` | Supprime un Saladier apres verification |
| `unlock_hidden_saladier` | `password` | Recherche dans les Saladiers caches (deniabilite plausible) |
| `get_saladier_attempts_info` | `uuid` | Retourne les tentatives echouees et restantes |

#### Feuilles (`commands/feuilles.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `create_feuille` | `saladierId`, `data`, `saladierPassword` | Cree une entree chiffree avec K_S |
| `get_feuille` | `uuid` | Dechiffre et retourne une entree |
| `list_feuilles` | `saladierId` | Liste toutes les entrees d'un Saladier |
| `update_feuille` | `uuid`, `data`, `saladierPassword` | Met a jour une entree |
| `delete_feuille` | `uuid`, `saladierPassword` | Supprime une entree |

#### Recuperation (`commands/recovery.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `generate_recovery_phrase` | - | Genere une phrase BIP39 de 12 mots |
| `recover_from_phrase` | `phrase` | Restaure la cle depuis la phrase |
| `check_recovery_status` | - | Verifie si le Kit de Secours a ete confirme |
| `confirm_recovery_saved` | - | Marque la phrase comme sauvegardee |

#### Parametres (`commands/settings.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `get_settings` | - | Charge les parametres utilisateur (JSON) |
| `save_settings` | `settings` | Sauvegarde les parametres |
| `apply_screenshot_protection` | `enabled` | Active/desactive la protection captures d'ecran |
| `update_last_activity` | - | Met a jour le timestamp d'activite |
| `get_inactivity_seconds` | - | Retourne les secondes depuis la derniere activite |
| `write_to_clipboard` | `text` | Ecrit dans le presse-papiers |
| `clear_clipboard` | - | Vide le presse-papiers |

#### Generateur (`commands/password_gen.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `generate_password` | `length`, `passwordType` | Genere un mot de passe aleatoire ou passphrase |

#### Import/Export (`commands/import_export.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `import_passwords` | donnees CSV/XML | Importe des mots de passe depuis un fichier |
| `export_encrypted_json` | - | Exporte tout en JSON chiffre |
| `export_csv_clear` | - | Exporte en CSV clair (necessite confirmation) |

#### Maintenance (`commands/maintenance.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `vacuum_database` | - | Optimise la taille du fichier SQLite |
| `check_integrity` | - | Verifie l'integrite de la base |

---

### Base de donnees

SQLite en mode WAL (Write-Ahead Logging) avec cles etrangeres activees.

#### Schema

```sql
-- Utilisateurs : un par compte, identifie par blind index
CREATE TABLE users (
    id                  TEXT PRIMARY KEY,  -- HMAC-SHA256(email)
    salt_master         BLOB NOT NULL,     -- 32 octets, sel Argon2id
    k_cloud_enc         BLOB NOT NULL,     -- nonce(24) + ciphertext (token verification)
    recovery_confirmed  INTEGER DEFAULT 0  -- 1 = Kit de Secours confirme
);

-- Saladiers : conteneurs chiffres independants
CREATE TABLE saladiers (
    uuid            TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id),
    name_enc        BLOB NOT NULL,     -- nom chiffre avec Master Key
    salt_saladier   BLOB NOT NULL,     -- 32 octets, sel pour K_S
    nonce           BLOB NOT NULL,     -- 24 octets, nonce pour name_enc
    verify_enc      BLOB NOT NULL,     -- token de verification chiffre avec K_S
    verify_nonce    BLOB NOT NULL,     -- 24 octets, nonce pour verify_enc
    hidden          INTEGER NOT NULL,  -- 0=visible, 1=cache (deniabilite plausible)
    failed_attempts INTEGER DEFAULT 0  -- compteur tentatives echouees
);

-- Feuilles : entrees (identifiants) dans un Saladier
CREATE TABLE feuilles (
    uuid        TEXT PRIMARY KEY,
    saladier_id TEXT NOT NULL REFERENCES saladiers(uuid) ON DELETE CASCADE,
    data_blob   BLOB NOT NULL,  -- JSON(FeuilleData) chiffre avec K_S
    nonce       BLOB NOT NULL   -- 24 octets
);

-- Parametres utilisateur (JSON)
CREATE TABLE settings (
    user_id TEXT PRIMARY KEY REFERENCES users(id),
    data    TEXT NOT NULL  -- JSON serialise de UserSettings
);
```

#### Modeles de donnees

**FeuilleData** (stocke chiffre dans `data_blob`) :
```rust
struct FeuilleData {
    title: String,
    username: String,
    password: String,
    url: String,
    notes: String,
}
```

**UserSettings** :
```rust
struct UserSettings {
    auto_lock_timeout: AutoLockTimeout,     // Immediate | After1Min | After5Min | Never
    auto_lock_on_sleep: bool,               // Verrouiller a la mise en veille
    auto_lock_on_close: bool,               // Verrouiller a la fermeture
    auto_lock_on_inactivity: bool,          // Verrouiller apres inactivite
    clipboard_clear_seconds: u32,           // Delai vidage presse-papiers (5-300s)
    screenshot_protection: bool,            // Bloquer captures d'ecran
    password_default_length: u32,           // Longueur par defaut (12-64)
    password_type: PasswordType,            // Alphanumeric | Passphrase
    favicon_policy: FaviconPolicy,          // None | ProxyAnonymous | Direct
    crash_reports: bool,                    // Rapports anonymes
    max_failed_attempts: u32,              // 0 = desactive, >0 = auto-destruction
    theme: Theme,                           // Dark | Light | System
    dead_man_switch_enabled: bool,          // Dead Man's Switch
    dead_man_switch_days: u32,              // Delai inactivite (7-365 jours)
    dead_man_switch_email: String,          // Contact de confiance
    clear_icon_cache_on_close: bool,        // Vider cache icones
}
```

---

### Composants frontend

#### Navigation

La navigation est geree par un enum `AppView` et des signaux Leptos (pas de router) :

```rust
enum AppView {
    Login,                                   // Ecran de connexion
    Register,                                // Creation de compte
    NagScreen,                               // Avertissement Kit de Secours
    Dashboard,                               // Liste des Saladiers
    SaladierUnlock { uuid, name },           // Mot de passe du Saladier
    SaladierView { uuid, name },             // Contenu du Saladier
    Recovery,                                // Phrase de recuperation
    Settings,                                // Page des parametres
}
```

#### Flux de navigation

```
Login ----------+---> NagScreen ---> Dashboard ---> SaladierUnlock ---> SaladierView
                |                     |
Register -------+                     +---> Settings
                                      +---> Recovery
```

#### Communication frontend/backend

Le frontend appelle le backend via `wasm_bindgen` :

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}
```

L'attribut `catch` est **obligatoire** pour intercepter les erreurs Tauri (`Result::Err`). Sans lui, les erreurs provoquent un panic WASM silencieux.

#### Plugins Tauri utilises

| Plugin | Namespace JS | Usage |
|--------|-------------|-------|
| `tauri-plugin-clipboard-manager` | `window.__TAURI__.clipboardManager` | Copier/vider presse-papiers |
| `tauri-plugin-dialog` | `window.__TAURI__.dialog` | Dialog systeme (deplacer cle) |

---

### Gestion de l'etat

#### Backend (`AppState`)

```rust
struct AppState {
    db: Mutex<Connection>,                          // Connexion SQLite
    session: Mutex<Option<Session>>,                // Session authentifiee
    saladier_keys: Mutex<HashMap<String, [u8; 32]>>, // Cache cles K_S
    data_dir: PathBuf,                              // Repertoire donnees
    last_activity: Mutex<Instant>,                  // Dernier evenement utilisateur
}
```

- **Session** : contient `user_id` (blind index) et `master_key_bytes` (32 octets, zeroise au drop)
- **Cache K_S** : les cles des Saladiers ouverts sont gardees en memoire pour eviter la re-derivation Argon2id a chaque operation
- **Activite** : `last_activity` est mis a jour par le frontend via `update_last_activity()` (throttle 5s)

#### Frontend (Signaux Leptos)

L'etat frontend est entierement gere par des signaux reactifs :

```rust
let (current_view, set_current_view) = signal(AppView::Login);
let (logged_in, set_logged_in) = signal(false);
let (user_settings, set_user_settings) = signal(Option::<UserSettings>::None);
```

Les transitions de vue sont declenchees par des `Effect::new()` qui reagissent aux changements de signaux.

---

### Fonctionnalites de securite

#### Verrouillage automatique

Apres connexion, le frontend :

1. **Charge les parametres** utilisateur
2. **Attache des listeners** `mousemove` et `keydown` (throttle 5s) qui appellent `update_last_activity()`
3. **Attache un listener** `visibilitychange` (proxy pour mise en veille) qui verrouille immediatement si active
4. **Lance un polling** toutes les 10s qui appelle `get_inactivity_seconds()` et compare avec le timeout configure

Au logout, tous les listeners et l'intervalle sont nettoyes.

#### Protection captures d'ecran

- **Au demarrage** : activee par defaut (`set_content_protected(true)`)
- **Apres login** : ajustee selon la preference utilisateur
- **Au logout** : re-activee (securite par defaut quand verrouille)
- **Toggle dynamique** : le changement dans les parametres s'applique immediatement

#### Vidage automatique du presse-papiers

Chaque Feuille affiche des boutons "Copier" pour l'identifiant et le mot de passe :

1. Le texte est copie via `clipboardManager.writeText()`
2. Un feedback "Copie !" s'affiche pendant 2 secondes
3. Un `Timeout` programme le vidage du presse-papiers apres N secondes (configurable, defaut 30s)

#### Auto-destruction des Saladiers

Si `max_failed_attempts > 0` :

1. Chaque tentative echouee incremente le compteur
2. Le frontend affiche le nombre de tentatives restantes
3. Si le maximum est atteint, le Saladier et toutes ses Feuilles sont supprimes
4. L'interface affiche un message de destruction et redirige vers le dashboard apres 3s

#### Deniabilite plausible

Les Saladiers marques `hidden = true` :

- N'apparaissent pas dans `list_saladiers()`
- Ne sont accessibles que via `unlock_hidden_saladier(password)` qui teste tous les Saladiers caches
- L'absence de resultat ne genere pas d'erreur (pour ne pas reveler leur existence)

#### Messages d'erreur generiques

L'enum `AppError` serialise des messages generiques vers le frontend :

```rust
InvalidCredentials | UserNotFound | DecryptionFailed  ->  "Identifiants invalides"
Database(_) | Io(_) | Internal(_)                      ->  "Erreur interne"
SaladierNotFound | FeuilleNotFound                     ->  "Ressource introuvable"
```

Cela empeche un attaquant de distinguer un email inexistant d'un mauvais mot de passe.

---

### Configuration

#### Parametres Tauri (`tauri.conf.json`)

```json
{
  "productName": "SaladVault",
  "version": "0.1.0",
  "identifier": "com.saladvault.app",
  "build": {
    "beforeDevCommand": "trunk serve --port 1420",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "trunk build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [{
      "title": "SaladVault - Gestionnaire de Mots de Passe",
      "width": 960, "height": 700,
      "minWidth": 600, "minHeight": 500
    }]
  }
}
```

#### Permissions plugins (`capabilities/default.json`)

Les permissions suivantes sont configurees :
- `dialog:allow-save` : ouverture du dialog systeme pour deplacer la cle
- `clipboard-manager:allow-write-text` : ecriture dans le presse-papiers

---

### Developpement

#### Prerequis

- [Rust](https://rustup.rs/) (edition 2021)
- [Trunk](https://trunkrs.dev/) : `cargo install trunk`
- Target WASM : `rustup target add wasm32-unknown-unknown`
- Dependances systeme Tauri : [documentation officielle](https://tauri.app/start/prerequisites/)

#### Lancer en mode developpement

```bash
cd rust-app
cargo tauri dev
```

Cela lance automatiquement `trunk serve` (frontend hot-reload sur le port 1420) et le backend Tauri.

#### Verifier la compilation

```bash
# Frontend (WASM)
cargo check --target wasm32-unknown-unknown

# Backend (natif)
cd src-tauri && cargo check
```

#### Lancer les tests

```bash
cd src-tauri && cargo test
```

---

### Build de production

```bash
cd rust-app
cargo tauri build
```

Genere les installateurs selon la plateforme :

| Plateforme | Format | Emplacement |
|-----------|--------|-------------|
| Windows | `.msi` | `src-tauri/target/release/bundle/msi/` |
| macOS | `.dmg` | `src-tauri/target/release/bundle/dmg/` |
| Linux | `.AppImage`, `.deb` | `src-tauri/target/release/bundle/` |

#### Donnees locales

L'application stocke ses donnees dans le repertoire standard de l'OS :

| OS | Chemin |
|----|--------|
| Windows | `%APPDATA%\com.saladvault.app\` |
| macOS | `~/Library/Application Support/com.saladvault.app/` |
| Linux | `~/.local/share/com.saladvault.app/` |

Fichiers :
- `saladvault.db` : base de donnees SQLite (blobs chiffres uniquement)
- `device_secret.key` : cle de peripherique (32 octets, deplacable vers stockage externe)

---

## :gb: English

### Table of contents

- [Architecture](#architecture-1)
- [Tech stack](#tech-stack)
- [Project structure](#project-structure)
- [Cryptographic model](#cryptographic-model)
- [Domain vocabulary](#domain-vocabulary)
- [Tauri commands](#tauri-commands)
- [Database](#database)
- [Frontend components](#frontend-components)
- [State management](#state-management)
- [Security features](#security-features)
- [Configuration](#configuration-1)
- [Development](#development)
- [Production build](#production-build)

---

### Architecture

```
+--------------------------------------------------+
|                    Tauri Webview                  |
|  +--------------------------------------------+  |
|  |         Frontend (Leptos 0.8 CSR)          |  |
|  |         Compiled to WASM via Trunk         |  |
|  |                                            |  |
|  |  App --> Login / Register / Dashboard       |  |
|  |         Settings / SaladierView            |  |
|  |         PanicUnlock / Recovery             |  |
|  +-------------------+------------------------+  |
|                      | invoke("command", args)    |
|  +-------------------v------------------------+  |
|  |          Backend (Tauri 2 + Rust)          |  |
|  |                                            |  |
|  |  Commands --> Crypto --> Database (SQLite)   |  |
|  |              |                             |  |
|  |              +-- XChaCha20-Poly1305        |  |
|  |              +-- Argon2id (OWASP)          |  |
|  |              +-- HKDF-SHA256               |  |
|  |              +-- HMAC-SHA256               |  |
|  +--------------------------------------------+  |
+--------------------------------------------------+
         |                          |
    device_secret.key          saladvault.db
    (32 bytes, local)        (encrypted blobs)
```

The frontend communicates with the backend exclusively via `wasm_bindgen` and Tauri's `invoke()` API. All cryptographic operations are performed on the backend side. The database only contains encrypted blobs.

---

### Tech stack

| Layer | Technology | Version |
|-------|------------|---------|
| Language | Rust | Edition 2021 |
| Desktop framework | Tauri | 2.x |
| UI framework | Leptos (CSR) | 0.8 |
| WASM compilation | Trunk | - |
| Local database | SQLite (WAL) | via `rusqlite` bundled |
| Symmetric encryption | XChaCha20-Poly1305 | `chacha20poly1305` 0.10 |
| KDF | Argon2id | `argon2` 0.5 |
| Key derivation | HKDF-SHA256 | `hkdf` 0.12 |
| Blind indexing | HMAC-SHA256 | `hmac` 0.12 + `sha2` 0.10 |
| Recovery phrase | BIP39 | `bip39` 2.x |
| QR Code | qrcodegen | 1.8 |
| Import/Export | CSV + XML | `csv` 1 + `quick-xml` 0.36 |

---

### Project structure

```
rust-app/
+-- Cargo.toml                  # Frontend manifest + workspace declaration
+-- index.html                  # HTML entry point (Trunk)
+-- styles.css                  # Stylesheet (dark theme, BEM-like)
+-- public/                     # Static assets
|
+-- src/                        # Leptos frontend (CSR/WASM)
|   +-- main.rs                 # Bootstrap: panic hook + mount_to_body
|   +-- app.rs                  # Root component, navigation, auto-lock
|   +-- components/
|       +-- mod.rs              # Module re-exports
|       +-- login.rs            # Login screen
|       +-- register.rs         # Account creation
|       +-- dashboard.rs        # Main hub (Saladier list)
|       +-- nag_screen.rs       # Recovery Kit backup warning
|       +-- recovery.rs         # BIP39 recovery phrase
|       +-- panic_unlock.rs     # Saladier unlock (Panic Mode)
|       +-- saladier_view.rs    # Feuille list + clipboard copy
|       +-- feuille_form.rs     # Entry create/edit form
|       +-- settings.rs         # Settings container + sub-navigation
|       +-- settings_security.rs    # Locking, clipboard, screenshots
|       +-- settings_keys.rs        # Key management, Recovery Kit, danger zone
|       +-- settings_devices.rs     # Devices, QR Code
|       +-- settings_saladiers.rs   # Saladier settings
|       +-- settings_data.rs        # Import/Export
|       +-- settings_privacy.rs     # Privacy, crash reports
|       +-- settings_general.rs     # Theme, generator, Dead Man's Switch
|       +-- settings_subscription.rs # Subscription (Freemium)
|       +-- password_utils.rs       # Password strength validation
|
+-- src-tauri/                  # Tauri backend (native)
    +-- Cargo.toml              # Backend manifest
    +-- tauri.conf.json         # Tauri configuration
    +-- capabilities/
    |   +-- default.json        # Plugin permissions (dialog, clipboard)
    +-- icons/                  # Application icons
    +-- src/
        +-- main.rs             # Entry point -> lib::run()
        +-- lib.rs              # Tauri setup, plugins, command registration
        +-- error.rs            # AppError enum (generic FR messages)
        +-- state.rs            # AppState: session, key cache, DB
        +-- commands/
        |   +-- mod.rs
        |   +-- auth.rs         # register, unlock, lock, verify, change pwd
        |   +-- device.rs       # init/check/move/export/regenerate device key
        |   +-- saladiers.rs    # CRUD Saladiers + failed attempts
        |   +-- feuilles.rs     # CRUD Feuilles (encrypted entries)
        |   +-- recovery.rs     # BIP39 phrase, restoration
        |   +-- settings.rs     # Settings, clipboard, activity, screenshots
        |   +-- password_gen.rs # Password generator
        |   +-- import_export.rs # CSV/XML import, JSON/CSV export
        |   +-- maintenance.rs  # VACUUM, integrity check
        +-- crypto/
        |   +-- mod.rs
        |   +-- keys.rs         # Key generation/loading, MasterKey reconstruction
        |   +-- xchacha.rs      # XChaCha20-Poly1305 encrypt/decrypt
        |   +-- argon2_kdf.rs   # Argon2id (OWASP params)
        |   +-- blind_index.rs  # HMAC-SHA256 for lookup without email leakage
        +-- db/
        |   +-- mod.rs          # open_database (SQLite WAL)
        |   +-- schema.rs       # CREATE TABLE (users, saladiers, feuilles, settings)
        |   +-- users.rs        # Users CRUD
        |   +-- saladiers.rs    # Saladiers CRUD + attempt counter
        |   +-- feuilles.rs     # Feuilles CRUD
        |   +-- settings.rs     # get/save settings (JSON)
        +-- models/
            +-- mod.rs
            +-- user.rs         # User struct
            +-- saladier.rs     # Saladier, SaladierInfo structs
            +-- feuille.rs      # Feuille, FeuilleData, FeuilleInfo structs
            +-- settings.rs     # UserSettings struct + enums
```

---

### Cryptographic model

#### Zero-Knowledge principle

No plaintext data is ever stored. The SQLite database only contains encrypted blobs. The server (future scope) will never see any password, email, or entry in cleartext.

#### Dual-Lock: Master Key reconstruction

The master key requires **two factors** to be reconstructed:

1. **Master password** (memorized by the user)
2. **Device key** (`device_secret.key`, 32 bytes, stored on the device)

```
Step 1: derived_key = Argon2id(password, salt_master)
            OWASP params: m=64MB, t=3, p=4, output=32 bytes

Step 2: prk = HKDF-Extract(salt=device_key, ikm=derived_key)

Step 3: master_key = HKDF-Expand(prk, info="SaladVault_MasterKey_v2", 32 bytes)

Step 4: Zeroize(derived_key)  // intermediate material erased
```

#### Data encryption

| Data | Key used | Algorithm |
|------|----------|-----------|
| Saladier names | Master Key | XChaCha20-Poly1305 (24-byte nonce) |
| Account verification token | Master Key | XChaCha20-Poly1305 |
| Saladier verification token | K_S (Saladier key) | XChaCha20-Poly1305 |
| Feuilles (entries) | K_S (Saladier key) | XChaCha20-Poly1305 |

#### K_S derivation (Saladier key)

Each Saladier has its own independently derived key:

```
K_S = Argon2id(saladier_password, salt_saladier)
    Params: m=64MB, t=3, p=4, output=32 bytes
```

#### Blind indexing

The user's email is never stored. A deterministic identifier is computed via:

```
user_id = HMAC-SHA256(
    key  = "SaladVault_BlindIndex_Pepper_v1_CHANGE_IN_PRODUCTION",
    msg  = normalize(email) + static_salt
)
```

Where `normalize(email) = email.trim().to_lowercase()`.

#### Memory zeroization

All cryptographic material is protected by `zeroize`:

- `MasterKey`: implements `Zeroize` + `Drop` (32 bytes erased on destruction)
- `Session`: zeroizes `master_key_bytes` and `user_id` on `Drop`
- `saladier_keys` cache: each K_S key is individually zeroized on logout

---

### Domain vocabulary

| Term | Meaning | Description |
|------|---------|-------------|
| **Potager** | User account | The user's global space |
| **Saladier** | Vault / Container | Isolated database, independently encrypted |
| **Feuille** | Entry | A username/password pair (+ URL, notes) |
| **Ingredient Secret** | Local key | `device_secret.key` file (32 bytes) |
| **Kit de Secours** | Recovery phrase | 12 BIP39 words to restore the local key |
| **Panic Mode** | Double lock | Each Saladier has its own password |

---

### Tauri commands

#### Authentication (`commands/auth.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `register` | `email`, `masterPassword` | Creates an account, generates the device key, establishes the session |
| `unlock` | `email`, `masterPassword` | Reconstructs the Master Key, verifies, establishes the session |
| `lock` | - | Clears the session and the Saladier key cache |
| `is_unlocked` | - | Returns `true` if a session is active |
| `verify_master_password` | `masterPassword` | Verifies the password without modifying the session |
| `change_master_password` | `currentPassword`, `newPassword` | Re-encrypts `k_cloud_enc` with the new key |

#### Device key (`commands/device.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `init_device_key` | - | Generates `device_secret.key` if absent |
| `check_device_key` | - | Checks that the key file exists |
| `get_device_key_path` | - | Returns the absolute path of the key |
| `move_device_key` | `newPath` | Moves the key (e.g., to a USB drive) |
| `export_device_key_qrcode` | - | Exports the key as base64 |
| `generate_device_key_qr_svg` | - | Generates an SVG QR code of the key |
| `regenerate_device_key` | `masterPassword` | Regenerates the key, re-encrypts all data |

#### Saladiers (`commands/saladiers.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `create_saladier` | `name`, `password`, `hidden` | Creates a Saladier with its own K_S key |
| `list_saladiers` | - | Lists visible Saladiers (decrypted names) |
| `open_saladier` | `uuid`, `password` | Unlocks a Saladier, caches K_S |
| `delete_saladier` | `uuid`, `masterPassword` | Deletes a Saladier after verification |
| `unlock_hidden_saladier` | `password` | Searches among hidden Saladiers (plausible deniability) |
| `get_saladier_attempts_info` | `uuid` | Returns failed and remaining attempts |

#### Feuilles (`commands/feuilles.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `create_feuille` | `saladierId`, `data`, `saladierPassword` | Creates an encrypted entry with K_S |
| `get_feuille` | `uuid` | Decrypts and returns an entry |
| `list_feuilles` | `saladierId` | Lists all entries of a Saladier |
| `update_feuille` | `uuid`, `data`, `saladierPassword` | Updates an entry |
| `delete_feuille` | `uuid`, `saladierPassword` | Deletes an entry |

#### Recovery (`commands/recovery.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `generate_recovery_phrase` | - | Generates a 12-word BIP39 phrase |
| `recover_from_phrase` | `phrase` | Restores the key from the phrase |
| `check_recovery_status` | - | Checks if the Recovery Kit has been confirmed |
| `confirm_recovery_saved` | - | Marks the phrase as saved |

#### Settings (`commands/settings.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `get_settings` | - | Loads user settings (JSON) |
| `save_settings` | `settings` | Saves settings |
| `apply_screenshot_protection` | `enabled` | Enables/disables screenshot protection |
| `update_last_activity` | - | Updates the activity timestamp |
| `get_inactivity_seconds` | - | Returns seconds since last activity |
| `write_to_clipboard` | `text` | Writes to the clipboard |
| `clear_clipboard` | - | Clears the clipboard |

#### Generator (`commands/password_gen.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `generate_password` | `length`, `passwordType` | Generates a random password or passphrase |

#### Import/Export (`commands/import_export.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `import_passwords` | CSV/XML data | Imports passwords from a file |
| `export_encrypted_json` | - | Exports everything as encrypted JSON |
| `export_csv_clear` | - | Exports as cleartext CSV (requires confirmation) |

#### Maintenance (`commands/maintenance.rs`)

| Command | Parameters | Description |
|---------|-----------|-------------|
| `vacuum_database` | - | Optimizes the SQLite file size |
| `check_integrity` | - | Verifies database integrity |

---

### Database

SQLite in WAL (Write-Ahead Logging) mode with foreign keys enabled.

#### Schema

```sql
-- Users: one per account, identified by blind index
CREATE TABLE users (
    id                  TEXT PRIMARY KEY,  -- HMAC-SHA256(email)
    salt_master         BLOB NOT NULL,     -- 32 bytes, Argon2id salt
    k_cloud_enc         BLOB NOT NULL,     -- nonce(24) + ciphertext (verification token)
    recovery_confirmed  INTEGER DEFAULT 0  -- 1 = Recovery Kit confirmed
);

-- Saladiers: independently encrypted containers
CREATE TABLE saladiers (
    uuid            TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id),
    name_enc        BLOB NOT NULL,     -- name encrypted with Master Key
    salt_saladier   BLOB NOT NULL,     -- 32 bytes, salt for K_S
    nonce           BLOB NOT NULL,     -- 24 bytes, nonce for name_enc
    verify_enc      BLOB NOT NULL,     -- verification token encrypted with K_S
    verify_nonce    BLOB NOT NULL,     -- 24 bytes, nonce for verify_enc
    hidden          INTEGER NOT NULL,  -- 0=visible, 1=hidden (plausible deniability)
    failed_attempts INTEGER DEFAULT 0  -- failed attempts counter
);

-- Feuilles: entries (credentials) in a Saladier
CREATE TABLE feuilles (
    uuid        TEXT PRIMARY KEY,
    saladier_id TEXT NOT NULL REFERENCES saladiers(uuid) ON DELETE CASCADE,
    data_blob   BLOB NOT NULL,  -- JSON(FeuilleData) encrypted with K_S
    nonce       BLOB NOT NULL   -- 24 bytes
);

-- User settings (JSON)
CREATE TABLE settings (
    user_id TEXT PRIMARY KEY REFERENCES users(id),
    data    TEXT NOT NULL  -- Serialized JSON of UserSettings
);
```

#### Data models

**FeuilleData** (stored encrypted in `data_blob`):
```rust
struct FeuilleData {
    title: String,
    username: String,
    password: String,
    url: String,
    notes: String,
}
```

**UserSettings**:
```rust
struct UserSettings {
    auto_lock_timeout: AutoLockTimeout,     // Immediate | After1Min | After5Min | Never
    auto_lock_on_sleep: bool,               // Lock on sleep
    auto_lock_on_close: bool,               // Lock on close
    auto_lock_on_inactivity: bool,          // Lock after inactivity
    clipboard_clear_seconds: u32,           // Clipboard clear delay (5-300s)
    screenshot_protection: bool,            // Block screenshots
    password_default_length: u32,           // Default length (12-64)
    password_type: PasswordType,            // Alphanumeric | Passphrase
    favicon_policy: FaviconPolicy,          // None | ProxyAnonymous | Direct
    crash_reports: bool,                    // Anonymous reports
    max_failed_attempts: u32,              // 0 = disabled, >0 = self-destruct
    theme: Theme,                           // Dark | Light | System
    dead_man_switch_enabled: bool,          // Dead Man's Switch
    dead_man_switch_days: u32,              // Inactivity delay (7-365 days)
    dead_man_switch_email: String,          // Trusted contact
    clear_icon_cache_on_close: bool,        // Clear icon cache
}
```

---

### Frontend components

#### Navigation

Navigation is managed by an `AppView` enum and Leptos signals (no router):

```rust
enum AppView {
    Login,                                   // Login screen
    Register,                                // Account creation
    NagScreen,                               // Recovery Kit backup warning
    Dashboard,                               // Saladier list
    SaladierUnlock { uuid, name },           // Saladier password
    SaladierView { uuid, name },             // Saladier contents
    Recovery,                                // Recovery phrase
    Settings,                                // Settings page
}
```

#### Navigation flow

```
Login ----------+---> NagScreen ---> Dashboard ---> SaladierUnlock ---> SaladierView
                |                     |
Register -------+                     +---> Settings
                                      +---> Recovery
```

#### Frontend/backend communication

The frontend calls the backend via `wasm_bindgen`:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}
```

The `catch` attribute is **mandatory** to intercept Tauri errors (`Result::Err`). Without it, errors cause a silent WASM panic.

#### Tauri plugins used

| Plugin | JS namespace | Usage |
|--------|-------------|-------|
| `tauri-plugin-clipboard-manager` | `window.__TAURI__.clipboardManager` | Copy/clear clipboard |
| `tauri-plugin-dialog` | `window.__TAURI__.dialog` | System dialog (move key) |

---

### State management

#### Backend (`AppState`)

```rust
struct AppState {
    db: Mutex<Connection>,                          // SQLite connection
    session: Mutex<Option<Session>>,                // Authenticated session
    saladier_keys: Mutex<HashMap<String, [u8; 32]>>, // K_S key cache
    data_dir: PathBuf,                              // Data directory
    last_activity: Mutex<Instant>,                  // Last user event
}
```

- **Session**: contains `user_id` (blind index) and `master_key_bytes` (32 bytes, zeroized on drop)
- **K_S cache**: keys of opened Saladiers are kept in memory to avoid re-deriving Argon2id on each operation
- **Activity**: `last_activity` is updated by the frontend via `update_last_activity()` (5s throttle)

#### Frontend (Leptos signals)

Frontend state is entirely managed by reactive signals:

```rust
let (current_view, set_current_view) = signal(AppView::Login);
let (logged_in, set_logged_in) = signal(false);
let (user_settings, set_user_settings) = signal(Option::<UserSettings>::None);
```

View transitions are triggered by `Effect::new()` that react to signal changes.

---

### Security features

#### Auto-lock

After login, the frontend:

1. **Loads user settings**
2. **Attaches listeners** for `mousemove` and `keydown` (5s throttle) that call `update_last_activity()`
3. **Attaches a listener** for `visibilitychange` (proxy for sleep) that locks immediately if enabled
4. **Starts polling** every 10s calling `get_inactivity_seconds()` and comparing with the configured timeout

On logout, all listeners and intervals are cleaned up.

#### Screenshot protection

- **On startup**: enabled by default (`set_content_protected(true)`)
- **After login**: adjusted according to user preference
- **On logout**: re-enabled (security by default when locked)
- **Dynamic toggle**: changes in settings apply immediately

#### Automatic clipboard clearing

Each Feuille displays "Copy" buttons for the username and password:

1. Text is copied via `clipboardManager.writeText()`
2. A "Copied!" feedback is displayed for 2 seconds
3. A `Timeout` schedules clipboard clearing after N seconds (configurable, default 30s)

#### Saladier self-destruct

If `max_failed_attempts > 0`:

1. Each failed attempt increments the counter
2. The frontend displays the number of remaining attempts
3. If the maximum is reached, the Saladier and all its Feuilles are deleted
4. The UI displays a destruction message and redirects to the dashboard after 3s

#### Plausible deniability

Saladiers marked `hidden = true`:

- Do not appear in `list_saladiers()`
- Are only accessible via `unlock_hidden_saladier(password)` which tests all hidden Saladiers
- The absence of a result does not generate an error (to avoid revealing their existence)

#### Generic error messages

The `AppError` enum serializes generic messages to the frontend:

```rust
InvalidCredentials | UserNotFound | DecryptionFailed  ->  "Identifiants invalides"
Database(_) | Io(_) | Internal(_)                      ->  "Erreur interne"
SaladierNotFound | FeuilleNotFound                     ->  "Ressource introuvable"
```

This prevents an attacker from distinguishing a non-existent email from a wrong password.

---

### Configuration

#### Tauri settings (`tauri.conf.json`)

```json
{
  "productName": "SaladVault",
  "version": "0.1.0",
  "identifier": "com.saladvault.app",
  "build": {
    "beforeDevCommand": "trunk serve --port 1420",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "trunk build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [{
      "title": "SaladVault - Gestionnaire de Mots de Passe",
      "width": 960, "height": 700,
      "minWidth": 600, "minHeight": 500
    }]
  }
}
```

#### Plugin permissions (`capabilities/default.json`)

The following permissions are configured:
- `dialog:allow-save`: opens the system dialog for moving the key
- `clipboard-manager:allow-write-text`: writes to the clipboard

---

### Development

#### Prerequisites

- [Rust](https://rustup.rs/) (edition 2021)
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- WASM target: `rustup target add wasm32-unknown-unknown`
- Tauri system dependencies: [official documentation](https://tauri.app/start/prerequisites/)

#### Run in development mode

```bash
cd rust-app
cargo tauri dev
```

This automatically launches `trunk serve` (frontend hot-reload on port 1420) and the Tauri backend.

#### Verify compilation

```bash
# Frontend (WASM)
cargo check --target wasm32-unknown-unknown

# Backend (native)
cd src-tauri && cargo check
```

#### Run tests

```bash
cd src-tauri && cargo test
```

---

### Production build

```bash
cd rust-app
cargo tauri build
```

Generates platform-specific installers:

| Platform | Format | Location |
|----------|--------|----------|
| Windows | `.msi` | `src-tauri/target/release/bundle/msi/` |
| macOS | `.dmg` | `src-tauri/target/release/bundle/dmg/` |
| Linux | `.AppImage`, `.deb` | `src-tauri/target/release/bundle/` |

#### Local data

The application stores its data in the OS standard directory:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\com.saladvault.app\` |
| macOS | `~/Library/Application Support/com.saladvault.app/` |
| Linux | `~/.local/share/com.saladvault.app/` |

Files:
- `saladvault.db`: SQLite database (encrypted blobs only)
- `device_secret.key`: device key (32 bytes, can be moved to external storage)
