# SaladVault - Application Desktop

Gestionnaire de mots de passe **Zero-Knowledge** a double verrouillage (Dual-Lock).
Client desktop construit avec **Tauri 2** + **Leptos 0.8** (CSR/WASM).

---

## Table des matieres

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

## Architecture

```
┌──────────────────────────────────────────────────┐
│                    Tauri Webview                  │
│  ┌────────────────────────────────────────────┐  │
│  │         Frontend (Leptos 0.8 CSR)          │  │
│  │         Compile en WASM via Trunk          │  │
│  │                                            │  │
│  │  App ─> Login / Register / Dashboard       │  │
│  │         Settings / SaladierView            │  │
│  │         PanicUnlock / Recovery             │  │
│  └───────────────┬────────────────────────────┘  │
│                  │ invoke("command", args)        │
│  ┌───────────────▼────────────────────────────┐  │
│  │          Backend (Tauri 2 + Rust)          │  │
│  │                                            │  │
│  │  Commands ─> Crypto ─> Database (SQLite)   │  │
│  │              │                             │  │
│  │              ├── XChaCha20-Poly1305        │  │
│  │              ├── Argon2id (OWASP)          │  │
│  │              ├── HKDF-SHA256               │  │
│  │              └── HMAC-SHA256               │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
         │                          │
    device_secret.key          saladvault.db
    (32 bytes, local)        (blobs chiffres)
```

Le frontend communique avec le backend exclusivement via `wasm_bindgen` et l'API `invoke()` de Tauri. Toutes les operations cryptographiques sont effectuees cote backend. La base de donnees ne contient que des blobs chiffres.

---

## Stack technique

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

## Structure du projet

```
rust-app/
├── Cargo.toml                  # Manifest frontend + declaration workspace
├── index.html                  # Point d'entree HTML (Trunk)
├── styles.css                  # Feuille de styles (theme sombre, BEM-like)
├── public/                     # Assets statiques
│
├── src/                        # Frontend Leptos (CSR/WASM)
│   ├── main.rs                 # Bootstrap : panic hook + mount_to_body
│   ├── app.rs                  # Composant racine, navigation, auto-lock
│   └── components/
│       ├── mod.rs              # Re-exports des modules
│       ├── login.rs            # Ecran de connexion
│       ├── register.rs         # Creation de compte
│       ├── dashboard.rs        # Hub principal (liste des Saladiers)
│       ├── nag_screen.rs       # Avertissement sauvegarde Kit de Secours
│       ├── recovery.rs         # Phrase de recuperation BIP39
│       ├── panic_unlock.rs     # Deverrouillage Saladier (Panic Mode)
│       ├── saladier_view.rs    # Vue des Feuilles + copie presse-papiers
│       ├── feuille_form.rs     # Formulaire creation/edition d'entree
│       ├── settings.rs         # Conteneur Parametres + sous-navigation
│       ├── settings_security.rs    # Verrouillage, presse-papiers, screenshots
│       ├── settings_keys.rs        # Gestion cles, Kit de Secours, zone danger
│       ├── settings_devices.rs     # Appareils, QR Code
│       ├── settings_saladiers.rs   # Parametres des Saladiers
│       ├── settings_data.rs        # Import/Export
│       ├── settings_privacy.rs     # Vie privee, rapports de crash
│       ├── settings_general.rs     # Theme, generateur, Dead Man's Switch
│       ├── settings_subscription.rs # Abonnement (Freemium)
│       └── password_utils.rs       # Validation force mot de passe
│
└── src-tauri/                  # Backend Tauri (natif)
    ├── Cargo.toml              # Manifest backend
    ├── tauri.conf.json         # Configuration Tauri
    ├── capabilities/
    │   └── default.json        # Permissions plugins (dialog, clipboard)
    ├── icons/                  # Icones application
    └── src/
        ├── main.rs             # Point d'entree → lib::run()
        ├── lib.rs              # Setup Tauri, plugins, enregistrement commandes
        ├── error.rs            # Enum AppError (messages generiques FR)
        ├── state.rs            # AppState : session, cache cles, DB
        ├── commands/
        │   ├── mod.rs
        │   ├── auth.rs         # register, unlock, lock, verify, change pwd
        │   ├── device.rs       # init/check/move/export/regenerate device key
        │   ├── saladiers.rs    # CRUD Saladiers + tentatives echouees
        │   ├── feuilles.rs     # CRUD Feuilles (entrees chiffrees)
        │   ├── recovery.rs     # Phrase BIP39, restauration
        │   ├── settings.rs     # Parametres, clipboard, activite, screenshots
        │   ├── password_gen.rs # Generateur de mots de passe
        │   ├── import_export.rs # Import CSV/XML, export JSON/CSV
        │   └── maintenance.rs  # VACUUM, integrity check
        ├── crypto/
        │   ├── mod.rs
        │   ├── keys.rs         # Generation/chargement cles, reconstruction MasterKey
        │   ├── xchacha.rs      # XChaCha20-Poly1305 encrypt/decrypt
        │   ├── argon2_kdf.rs   # Argon2id (params OWASP)
        │   └── blind_index.rs  # HMAC-SHA256 pour lookup sans fuite d'email
        ├── db/
        │   ├── mod.rs          # open_database (SQLite WAL)
        │   ├── schema.rs       # CREATE TABLE (users, saladiers, feuilles, settings)
        │   ├── users.rs        # CRUD utilisateurs
        │   ├── saladiers.rs    # CRUD Saladiers + compteur tentatives
        │   ├── feuilles.rs     # CRUD Feuilles
        │   └── settings.rs     # get/save settings (JSON)
        └── models/
            ├── mod.rs
            ├── user.rs         # Struct User
            ├── saladier.rs     # Structs Saladier, SaladierInfo
            ├── feuille.rs      # Structs Feuille, FeuilleData, FeuilleInfo
            └── settings.rs     # Struct UserSettings + enums
```

---

## Modele cryptographique

### Principe Zero-Knowledge

Aucune donnee en clair n'est jamais stockee. La base de donnees SQLite ne contient que des blobs chiffres. Le serveur ne voit jamais de mot de passe, d'email ou d'entree en clair.

### Dual-Lock : Reconstruction de la Master Key

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

### Chiffrement des donnees

| Donnee | Cle utilisee | Algorithme |
|--------|-------------|------------|
| Noms de Saladiers | Master Key | XChaCha20-Poly1305 (nonce 24 octets) |
| Token de verification compte | Master Key | XChaCha20-Poly1305 |
| Token de verification Saladier | K_S (cle Saladier) | XChaCha20-Poly1305 |
| Feuilles (entrees) | K_S (cle Saladier) | XChaCha20-Poly1305 |

### Derivation de K_S (cle de Saladier)

Chaque Saladier possede sa propre cle derivee independamment :

```
K_S = Argon2id(saladier_password, salt_saladier)
    Params : m=64MB, t=3, p=4, output=32 bytes
```

### Indexation aveugle (Blind Index)

L'email de l'utilisateur n'est jamais stocke. Un identifiant deterministe est calcule via :

```
user_id = HMAC-SHA256(
    key  = "SaladVault_BlindIndex_Pepper_v1_CHANGE_IN_PRODUCTION",
    msg  = normalize(email) + static_salt
)
```

Ou `normalize(email) = email.trim().to_lowercase()`.

### Zeroisage memoire

Tout materiel cryptographique est protege par `zeroize` :

- `MasterKey` : implemente `Zeroize` + `Drop` (32 octets effaces a la destruction)
- `Session` : zeroise `master_key_bytes` et `user_id` au `Drop`
- Cache `saladier_keys` : chaque cle K_S zeroisee individuellement au logout

---

## Vocabulaire metier

| Terme | Signification | Description |
|-------|--------------|-------------|
| **Potager** | Compte utilisateur | L'espace global de l'utilisateur |
| **Saladier** | Vault / Conteneur | Base de donnees isolee, chiffree independamment |
| **Feuille** | Entree | Un couple identifiant/mot de passe (+ URL, notes) |
| **Ingredient Secret** | Cle locale | Fichier `device_secret.key` (32 octets) |
| **Kit de Secours** | Phrase de recuperation | 12 mots BIP39 pour restaurer la cle locale |
| **Panic Mode** | Double verrouillage | Chaque Saladier a son propre mot de passe |

---

## Commandes Tauri

### Authentification (`commands/auth.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `register` | `email`, `masterPassword` | Cree un compte, genere la device key, etablit la session |
| `unlock` | `email`, `masterPassword` | Reconstruit la Master Key, verifie, etablit la session |
| `lock` | - | Efface la session et le cache des cles Saladier |
| `is_unlocked` | - | Retourne `true` si une session est active |
| `verify_master_password` | `masterPassword` | Verifie le mot de passe sans modifier la session |
| `change_master_password` | `currentPassword`, `newPassword` | Re-chiffre `k_cloud_enc` avec la nouvelle cle |

### Cle de peripherique (`commands/device.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `init_device_key` | - | Genere `device_secret.key` si absent |
| `check_device_key` | - | Verifie l'existence du fichier cle |
| `get_device_key_path` | - | Retourne le chemin absolu de la cle |
| `move_device_key` | `newPath` | Deplace la cle (ex: vers une cle USB) |
| `export_device_key_qrcode` | - | Exporte la cle en base64 |
| `generate_device_key_qr_svg` | - | Genere un QR code SVG de la cle |
| `regenerate_device_key` | `masterPassword` | Regenere la cle, re-chiffre toutes les donnees |

### Saladiers (`commands/saladiers.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `create_saladier` | `name`, `password`, `hidden` | Cree un Saladier avec sa propre cle K_S |
| `list_saladiers` | - | Liste les Saladiers visibles (noms dechiffres) |
| `open_saladier` | `uuid`, `password` | Deverrouille un Saladier, met K_S en cache |
| `delete_saladier` | `uuid`, `masterPassword` | Supprime un Saladier apres verification |
| `unlock_hidden_saladier` | `password` | Recherche dans les Saladiers caches (deniabilite plausible) |
| `get_saladier_attempts_info` | `uuid` | Retourne les tentatives echouees et restantes |

### Feuilles (`commands/feuilles.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `create_feuille` | `saladierId`, `data`, `saladierPassword` | Cree une entree chiffree avec K_S |
| `get_feuille` | `uuid` | Dechiffre et retourne une entree |
| `list_feuilles` | `saladierId` | Liste toutes les entrees d'un Saladier |
| `update_feuille` | `uuid`, `data`, `saladierPassword` | Met a jour une entree |
| `delete_feuille` | `uuid`, `saladierPassword` | Supprime une entree |

### Recuperation (`commands/recovery.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `generate_recovery_phrase` | - | Genere une phrase BIP39 de 12 mots |
| `recover_from_phrase` | `phrase` | Restaure la cle depuis la phrase |
| `check_recovery_status` | - | Verifie si le Kit de Secours a ete confirme |
| `confirm_recovery_saved` | - | Marque la phrase comme sauvegardee |

### Parametres (`commands/settings.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `get_settings` | - | Charge les parametres utilisateur (JSON) |
| `save_settings` | `settings` | Sauvegarde les parametres |
| `apply_screenshot_protection` | `enabled` | Active/desactive la protection captures d'ecran |
| `update_last_activity` | - | Met a jour le timestamp d'activite |
| `get_inactivity_seconds` | - | Retourne les secondes depuis la derniere activite |
| `write_to_clipboard` | `text` | Ecrit dans le presse-papiers |
| `clear_clipboard` | - | Vide le presse-papiers |

### Generateur (`commands/password_gen.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `generate_password` | `length`, `passwordType` | Genere un mot de passe aleatoire ou passphrase |

### Import/Export (`commands/import_export.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `import_passwords` | donnees CSV/XML | Importe des mots de passe depuis un fichier |
| `export_encrypted_json` | - | Exporte tout en JSON chiffre |
| `export_csv_clear` | - | Exporte en CSV clair (necessite confirmation) |

### Maintenance (`commands/maintenance.rs`)

| Commande | Parametres | Description |
|----------|-----------|-------------|
| `vacuum_database` | - | Optimise la taille du fichier SQLite |
| `check_integrity` | - | Verifie l'integrite de la base |

---

## Base de donnees

SQLite en mode WAL (Write-Ahead Logging) avec cles etrangeres activees.

### Schema

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

### Modeles de donnees

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

## Composants frontend

### Navigation

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

### Flux de navigation

```
Login ──────┬──> NagScreen ──> Dashboard ──> SaladierUnlock ──> SaladierView
            │                     │
Register ───┘                     ├──> Settings
                                  └──> Recovery
```

### Communication frontend/backend

Le frontend appelle le backend via `wasm_bindgen` :

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}
```

L'attribut `catch` est **obligatoire** pour intercepter les erreurs Tauri (`Result::Err`). Sans lui, les erreurs provoquent un panic WASM silencieux.

### Plugins Tauri utilises

| Plugin | Namespace JS | Usage |
|--------|-------------|-------|
| `tauri-plugin-clipboard-manager` | `window.__TAURI__.clipboardManager` | Copier/vider presse-papiers |
| `tauri-plugin-dialog` | `window.__TAURI__.dialog` | Dialog systeme (deplacer cle) |

---

## Gestion de l'etat

### Backend (`AppState`)

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

### Frontend (Signaux Leptos)

L'etat frontend est entierement gere par des signaux reactifs :

```rust
let (current_view, set_current_view) = signal(AppView::Login);
let (logged_in, set_logged_in) = signal(false);
let (user_settings, set_user_settings) = signal(Option::<UserSettings>::None);
```

Les transitions de vue sont declenchees par des `Effect::new()` qui reagissent aux changements de signaux.

---

## Fonctionnalites de securite

### Verrouillage automatique

Apres connexion, le frontend :

1. **Charge les parametres** utilisateur
2. **Attache des listeners** `mousemove` et `keydown` (throttle 5s) qui appellent `update_last_activity()`
3. **Attache un listener** `visibilitychange` (proxy pour mise en veille) qui verrouille immediatement si active
4. **Lance un polling** toutes les 10s qui appelle `get_inactivity_seconds()` et compare avec le timeout configure

Au logout, tous les listeners et l'intervalle sont nettoyes.

### Protection captures d'ecran

- **Au demarrage** : activee par defaut (`set_content_protected(true)`)
- **Apres login** : ajustee selon la preference utilisateur
- **Au logout** : re-activee (securite par defaut quand verrouille)
- **Toggle dynamique** : le changement dans les parametres s'applique immediatement

### Vidage automatique du presse-papiers

Chaque Feuille affiche des boutons "Copier" pour l'identifiant et le mot de passe :

1. Le texte est copie via `clipboardManager.writeText()`
2. Un feedback "Copie !" s'affiche pendant 2 secondes
3. Un `Timeout` programme le vidage du presse-papiers apres N secondes (configurable, defaut 30s)

### Auto-destruction des Saladiers

Si `max_failed_attempts > 0` :

1. Chaque tentative echouee incremente le compteur
2. Le frontend affiche le nombre de tentatives restantes
3. Si le maximum est atteint, le Saladier et toutes ses Feuilles sont supprimes
4. L'interface affiche un message de destruction et redirige vers le dashboard apres 3s

### Deniabilite plausible

Les Saladiers marques `hidden = true` :

- N'apparaissent pas dans `list_saladiers()`
- Ne sont accessibles que via `unlock_hidden_saladier(password)` qui teste tous les Saladiers caches
- L'absence de resultat ne genere pas d'erreur (pour ne pas reveler leur existence)

### Messages d'erreur generiques

L'enum `AppError` serialise des messages generiques vers le frontend :

```rust
InvalidCredentials | UserNotFound | DecryptionFailed  →  "Identifiants invalides"
Database(_) | Io(_) | Internal(_)                      →  "Erreur interne"
SaladierNotFound | FeuilleNotFound                     →  "Ressource introuvable"
```

Cela empeche un attaquant de distinguer un email inexistant d'un mauvais mot de passe.

---

## Configuration

### Parametres Tauri (`tauri.conf.json`)

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

### Permissions plugins (`capabilities/default.json`)

Les permissions suivantes sont configurees :
- `dialog:allow-save` : ouverture du dialog systeme pour deplacer la cle
- `clipboard-manager:allow-write-text` : ecriture dans le presse-papiers

---

## Developpement

### Prerequis

- [Rust](https://rustup.rs/) (edition 2021)
- [Trunk](https://trunkrs.dev/) : `cargo install trunk`
- Target WASM : `rustup target add wasm32-unknown-unknown`
- Dependances systeme Tauri : [documentation officielle](https://tauri.app/start/prerequisites/)

### Lancer en mode developpement

```bash
cd rust-app
cargo tauri dev
```

Cela lance automatiquement `trunk serve` (frontend hot-reload sur le port 1420) et le backend Tauri.

### Verifier la compilation

```bash
# Frontend (WASM)
cargo check --target wasm32-unknown-unknown

# Backend (natif)
cd src-tauri && cargo check
```

### Lancer les tests

```bash
cd src-tauri && cargo test
```

---

## Build de production

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

### Donnees locales

L'application stocke ses donnees dans le repertoire standard de l'OS :

| OS | Chemin |
|----|--------|
| Windows | `%APPDATA%\com.saladvault.app\` |
| macOS | `~/Library/Application Support/com.saladvault.app/` |
| Linux | `~/.local/share/com.saladvault.app/` |

Fichiers :
- `saladvault.db` : base de donnees SQLite (blobs chiffres uniquement)
- `device_secret.key` : cle de peripherique (32 octets, deplacable vers stockage externe)
