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
