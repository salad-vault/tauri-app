# Issues à traiter

## 1. Bouton de changement de langue mal positionné

Le bouton de changement de langue (`lang-toggle`) est actuellement placé de manière isolée en haut de l'app, en dehors du flux des autres boutons. Il faudrait le déplacer au même niveau que les autres boutons d'action (header du dashboard, header des settings, etc.) pour qu'il soit cohérent avec le reste de l'interface.

**Fichiers concernés :** `src/app.rs`, `styles.css`

## 2. Améliorer le visuel de la page Guide / Documentation

La page Documentation (`documentation.rs`) utilise des classes CSS custom (`doc-content`, `doc-section`, `doc-section-header`, `doc-chevron`, etc.) qui ne sont pas définies dans `styles.css`. Le composant s'affiche donc sans style et ne correspond pas au design system de l'application (cards, settings-group, boutons ghost, etc.).

**Fichiers concernés :** `src/components/documentation.rs`, `styles.css`
