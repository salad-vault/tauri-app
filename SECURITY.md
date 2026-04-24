# Security Policy / Politique de securite

---

## Table of contents / Table des matieres

- [Francais](#-francais)
- [English](#-english)

---

## :fr: Francais

### Versions supportees

| Version | Supportee          |
| ------- | ------------------ |
| latest  | :white_check_mark: |

### Signaler une vulnerabilite

Si vous decouvrez une vulnerabilite de securite dans SaladVault, veuillez la signaler de maniere responsable.

**Email :** [security@saladvault.com](mailto:security@saladvault.com)

#### Ce qu'il faut inclure

- Description de la vulnerabilite
- Etapes pour reproduire le probleme
- Impact potentiel
- Correctif suggere (le cas echeant)

#### Ce a quoi vous attendre

- **Accuse de reception** sous 48 heures
- **Evaluation** sous 7 jours
- **Objectif de resolution** sous 90 jours (selon la gravite)
- Mention dans les notes de version (sauf si vous preferez rester anonyme)

#### Perimetre

- Application desktop SaladVault (Tauri)
- Serveur API SaladVault
- Implementations cryptographiques (Argon2id, XChaCha20-Poly1305, HKDF, HMAC)
- Authentification et gestion de session
- Stockage des donnees et chiffrement au repos

#### Hors perimetre

- Attaques par ingenierie sociale
- Attaques par deni de service
- Vulnerabilites dans les dependances tierces (signalez-les en amont, mais informez-nous)

#### Sphre de securite (Safe Harbor)

Nous considerons que la recherche en securite menee de bonne foi est autorisee. Nous ne poursuivrons pas en justice les chercheurs qui :

- Agissent de bonne foi pour eviter les atteintes a la vie privee, la destruction de donnees et l'interruption de service
- N'interagissent qu'avec des comptes qui leur appartiennent ou avec une autorisation explicite
- Signalent les vulnerabilites rapidement et ne les exploitent pas au-dela de la preuve de concept

**Veuillez NE PAS ouvrir une issue publique sur GitHub pour les vulnerabilites de securite.**

### Limites connues du modele Zero-Knowledge en open-source

SaladVault est open-source. Certaines valeurs qualifiees de "pepper" ou "secret compile-time" dans le code
(notamment `PEPPER_SEED` dans `src-tauri/src/crypto/blind_index.rs`) sont par definition publiques
puisqu'elles sont visibles dans le code source.

**Consequence :** en cas de fuite de la base de donnees serveur (`sync_vaults`, `server_users`...), un
attaquant qui dispose du code source peut recomputer les `blind_id` a partir d'un dictionnaire d'emails
(attaque par enumeration sur l'espace email). Cela permet d'associer un email a un compte SaladVault,
mais **pas** de dechiffrer les donnees vault.

**Protection effective :** le contenu des Saladiers et des Feuilles reste illisible car chiffre par
`master_key = HKDF(device_key, Argon2id(master_password, salt))`. L'attaquant aurait besoin
simultanement de :

1. la base de donnees serveur (blobs chiffres)
2. le fichier `device_secret.key` (present uniquement sur l'appareil de l'utilisateur)
3. le `master_password` de l'utilisateur (jamais transmis, jamais stocke)

Le pepper public protege donc uniquement contre l'**enumeration triviale** des comptes via le
`blind_id`, pas contre la confidentialite des donnees.

**Evolution future possible :** introduire un pepper serveur via variable d'environnement
(`PEPPER_SERVER_KEY`) applique en HMAC supplementaire avant stockage. Il s'agit d'un breaking
change (tous les comptes existants seraient invalides) qui sera traite lors d'un audit securite
externe avant v1.0.

---

## :gb: English

### Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |

### Reporting a Vulnerability

If you discover a security vulnerability in SaladVault, please report it responsibly.

**Email:** [security@saladvault.com](mailto:security@saladvault.com)

#### What to include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

#### What to expect

- **Acknowledgment** within 48 hours
- **Assessment** within 7 days
- **Resolution target** within 90 days (depending on severity)
- Credit in the release notes (unless you prefer anonymity)

#### Scope

- SaladVault desktop application (Tauri)
- SaladVault API server
- Cryptographic implementations (Argon2id, XChaCha20-Poly1305, HKDF, HMAC)
- Authentication and session management
- Data storage and encryption at rest

#### Out of scope

- Social engineering attacks
- Denial of service attacks
- Vulnerabilities in third-party dependencies (report these upstream, but let us know)

#### Safe Harbor

We consider security research conducted in good faith to be authorized. We will not pursue legal action against researchers who:

- Act in good faith to avoid privacy violations, data destruction, and service disruption
- Only interact with accounts they own or with explicit permission
- Report vulnerabilities promptly and do not exploit them beyond proof of concept

**Please do NOT open a public GitHub issue for security vulnerabilities.**

### Known limits of open-source Zero-Knowledge model

SaladVault is open-source. Values labelled as "pepper" or "compile-time secret" in the code (notably
`PEPPER_SEED` in `src-tauri/src/crypto/blind_index.rs`) are **not actually secret** since the source
code is public.

**Consequence:** if the server database leaks (`sync_vaults`, `server_users`), an attacker with access
to the source code can recompute `blind_id` values from a dictionary of email addresses (email-space
enumeration). This allows linking an email to a SaladVault account, but **does not** decrypt vault
data.

**Effective protection:** the content of Saladiers and Feuilles remains unreadable because it is
encrypted with `master_key = HKDF(device_key, Argon2id(master_password, salt))`. An attacker would
need all three:

1. the server database (encrypted blobs)
2. the `device_secret.key` file (stored only on the user's device)
3. the user's `master_password` (never transmitted, never stored)

The public pepper therefore only protects against **trivial account enumeration** via `blind_id`, not
against data confidentiality.

**Possible future evolution:** introduce a server-side pepper via environment variable
(`PEPPER_SERVER_KEY`) applied as an additional HMAC before storage. This would be a breaking change
(all existing accounts would be invalidated) and is planned alongside an external security audit
before v1.0.
