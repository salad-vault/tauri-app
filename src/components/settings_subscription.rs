use leptos::prelude::*;

#[component]
pub fn SettingsSubscription() -> impl IntoView {
    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">"💎 Abonnement"</h2>

            // Current plan banner
            <div class="subscription-current-plan">
                <div class="subscription-current-info">
                    <span class="subscription-plan-name">"Jardinier"</span>
                    <span class="subscription-plan-price">"Gratuit"</span>
                </div>
                <span class="pricing-badge pricing-badge-active">"Plan actuel"</span>
            </div>

            // 4-column pricing grid
            <div class="pricing-grid">
                // Card 1 — Jardinier (free, current)
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <span class="pricing-tier-icon">"🌱"</span>
                        <div class="pricing-tier-name">"Jardinier"</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">"Gratuit"</span>
                        </div>
                        <div class="pricing-tier-subtitle">"La sécurité sans compromis"</div>
                    </div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Chiffrement Zero-Knowledge"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Dual-Lock Protocol"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Saladiers et Feuilles illimités"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Générateur de mots de passe"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Kit de Secours BIP39"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Verrouillage auto + protection screenshots"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Vidage auto du presse-papiers"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Import / Export"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Dead Man\u{2019}s Switch (local)"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Destruction par tentatives"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"1 appareil"</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-current">"Plan actuel"</button>
                </div>

                // Card 2 — Maraîcher (recommended)
                <div class="pricing-card pricing-card-highlighted">
                    <span class="pricing-badge pricing-badge-recommended">"Recommandé"</span>
                    <div class="pricing-card-header">
                        <span class="pricing-tier-icon">"🥬"</span>
                        <div class="pricing-tier-name">"Maraîcher"</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">"3,99 €"</span>
                            <span class="pricing-period">"/ mois"</span>
                        </div>
                        <div class="pricing-tier-subtitle">"Le confort au quotidien"</div>
                    </div>

                    <div class="pricing-divider">"Tout Jardinier, plus :"</div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Sync jusqu\u{2019}à 5 appareils"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Extension navigateur (autofill)"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Sentinelle (alertes fuites)"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"TOTP intégré (2FA)"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Partage sécurisé à usage unique"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Pièces jointes chiffrées (10 Mo)"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Favoris, tags, historique MdP"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Thèmes personnalisés"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Support prioritaire"</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-soon">"Bientôt disponible"</button>
                </div>

                // Card 3 — Potager Familial
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <span class="pricing-tier-icon">"👨\u{200d}👩\u{200d}👧\u{200d}👦"</span>
                        <div class="pricing-tier-name">"Potager Familial"</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">"6,99 €"</span>
                            <span class="pricing-period">"/ mois"</span>
                        </div>
                        <div class="pricing-tier-subtitle">"La sécurité pour toute la famille"</div>
                    </div>

                    <div class="pricing-divider">"Tout Maraîcher, plus :"</div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"6 comptes utilisateurs"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Saladiers partagés (famille)"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Rôles et permissions"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Tableau de bord admin"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Appareils illimités"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Dead Man\u{2019}s Switch actif (serveur)"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Héritage numérique"</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-soon">"Bientôt disponible"</button>
                </div>

                // Card 4 — Exploitation (enterprise)
                <div class="pricing-card">
                    <div class="pricing-card-header">
                        <span class="pricing-tier-icon">"🏢"</span>
                        <div class="pricing-tier-name">"Exploitation"</div>
                        <div class="pricing-tier-price">
                            <span class="pricing-amount">"Sur devis"</span>
                        </div>
                        <div class="pricing-tier-subtitle">"Pour les équipes et organisations"</div>
                    </div>

                    <div class="pricing-divider">"Tout Familial, plus :"</div>

                    <ul class="pricing-features">
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Utilisateurs illimités"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"SSO / SAML / LDAP"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Politiques d\u{2019}entreprise"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Journal d\u{2019}audit chiffré"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Groupes et collections"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Provisioning SCIM"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Rotation automatique des MdP"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"Auto-hébergement (self-hosted)"</span>
                        </li>
                        <li class="pricing-feature">
                            <span class="pricing-check">"✓"</span>
                            <span>"SLA 99.9%, API développeur"</span>
                        </li>
                    </ul>

                    <button class="pricing-btn pricing-btn-soon">"Bientôt disponible"</button>
                </div>
            </div>

            // Footer note
            <div class="subscription-footer-note">
                <p>"Les abonnements seront disponibles avec la version serveur. Votre plan gratuit reste pleinement fonctionnel en local."</p>
            </div>
        </div>
    }
}
