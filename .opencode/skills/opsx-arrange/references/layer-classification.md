# Layer Classification Rules (Arrange)

Every task must be classified into exactly one of eight dependency layers.
Classification determines build order: layer 0 tasks first, layer 7 last.

---

## Classification Method

For each task, examine:
1. **Task description text** — what it builds or sets up
2. **Context from design.md and specs/** — what role the component plays
3. **Referenced technologies** — what layer those technologies belong to
4. **Dependencies** — what must exist before this task can run

Match against the keyword and pattern tables below. If a task matches multiple
layers, classify it at the **lowest** layer it touches (foundation wins).

If no layer matches, classify as "unclassified" and ask the user.

---

## Layer 0: Infrastructure

The foundational runtime and tooling everything else runs on.

**Keywords:**
- database, db, postgres, mysql, sqlite, mongodb, redis, dynamodb, firestore
- server, web server, http server, app server, express, fastify, actix, axum, flask, django
- docker, container, kubernetes, k8s, pod, deployment, deploy
- build, build system, webpack, vite, cargo, make, cmake, bundler
- configuration, config, environment, env, dotenv
- ci/cd, pipeline, github actions, workflow
- hosting, domain, dns, ssl, tls, certificate
- project setup, initialize, scaffold, bootstrap, boilerplate

**Patterns:**
- "Set up X", "Initialize X", "Create X config", "Configure X"
- Tasks that produce a runnable but empty system
- Tasks that install dependencies or configure tooling
- Tasks that create database connections or server instances

**Examples:**
- "Set up PostgreSQL database"
- "Initialize Express server with TypeScript"
- "Configure Docker development environment"
- "Set up Vite build pipeline"
- "Create project configuration files"

---

## Layer 1: Core Data

Data structures, schemas, and the code that reads/writes persistent storage.

**Keywords:**
- model, schema, entity, type, struct, interface (data shape)
- migration, migrate, seed, fixture
- repository, repo, data access, dao, store, persistence
- orm, prisma, sequelize, sqlalchemy, diesel, typeorm, drizzle
- table, collection, document, record, row
- validation (data shape), constraint, index, unique, foreign key
- crud, create/read/update/delete (at data layer)

**Patterns:**
- "Create X model/schema", "Define X type", "Add migration for X"
- Tasks that define what data looks like
- Tasks that set up data access functions (not business logic)
- Tasks that create database tables or collections

**Examples:**
- "Create User model with email, password hash, timestamps"
- "Add migration for sessions table"
- "Set up UserRepository with CRUD operations"
- "Define TypeScript interfaces for API responses"
- "Create Prisma schema for all entities"

---

## Layer 2: Security/Auth

Authentication, authorization, encryption, and access control.

**Keywords:**
- auth, authentication, login, logout, sign in, sign up, sign out
- authorization, permission, role, rbac, acl, access control, policy
- session, token, jwt, cookie, session management
- password, hash, bcrypt, scrypt, argon2, salt
- encrypt, decrypt, cryptography, ssl, tls, certificate (for auth)
- oauth, sso, saml, openid, oidc, provider
- middleware (auth-specific), guard, protector, shield
- csrf, cors, xss, sanitization, rate limit (security-related)

**Patterns:**
- "Add authentication for X", "Implement login/signup"
- Tasks that verify identity or check permissions
- Tasks that create auth middleware or guards
- Tasks that handle sessions or tokens

**Examples:**
- "Implement JWT authentication middleware"
- "Add login and registration endpoints"
- "Set up role-based access control"
- "Create session management with Redis"
- "Add OAuth2 integration with Google"

---

## Layer 3: Core Services

Business logic, internal APIs, and the glue between data and presentation.

**Keywords:**
- service, business logic, domain, use case, interactor, handler
- api, endpoint, route, controller, resolver, action
- middleware (non-auth), pipeline, filter, interceptor
- validation (business rules), sanitize (input), transform
- event, event handler, listener, subscriber, publisher, emitter
- queue, job, worker, background task, async processing
- email, notification, alert, webhook (outgoing)
- search, filter, sort, paginate, aggregation
- logic, calculation, algorithm, processing, transformation

**Patterns:**
- "Implement X service", "Add endpoint for X", "Create X business logic"
- Tasks that coordinate between data layer and external interfaces
- Tasks that implement business rules or validation logic
- Tasks that create internal or external-facing APIs

**Examples:**
- "Implement UserService with profile management"
- "Create REST API endpoints for user CRUD"
- "Add input validation middleware for all routes"
- "Implement email notification service"
- "Create background job processor for report generation"

---

## Layer 4: External Integration

Connections to third-party services and external systems.

**Keywords:**
- third-party, external, integration, sdk, client library
- stripe, paypal, payment, billing, subscription
- aws, gcp, azure, cloud, s3, sqs, sns, cloudfront
- sendgrid, mailgun, twilio, sms, push notification
- analytics, tracking, metrics, telemetry, sentry, datadog
- webhooks (incoming), callback, integration point
- graphql client, rest client, api client, http client
- import, export, sync, migration (from external system)

**Patterns:**
- "Integrate with X", "Add X client", "Connect to X API"
- Tasks that call external services
- Tasks that receive webhooks from external systems
- Tasks that sync data with external sources

**Examples:**
- "Integrate Stripe payment processing"
- "Add SendGrid email client"
- "Set up AWS S3 file upload"
- "Create webhook receiver for GitHub events"
- "Implement Google Analytics tracking"

---

## Layer 5: Client Framework

Frontend scaffolding, state management, routing, and shared UI infrastructure.

**Keywords:**
- frontend, client, ui, react, vue, angular, svelte, next, nuxt
- state, store, redux, zustand, mobx, context, provider
- router, route, navigation, link, history
- component library, design system, theme, style, css, tailwind
- layout, shell, scaffold, wrapper, container, page template
- hook, composable, util (frontend), helper (frontend)
- asset, image, icon, font, static file
- i18n, localization, translation, locale

**Patterns:**
- "Set up X framework", "Configure X state management", "Create layout X"
- Tasks that establish the frontend architecture
- Tasks that create reusable infrastructure components
- Tasks that configure routing or state management

**Examples:**
- "Initialize React app with TypeScript"
- "Set up Redux store with slices"
- "Create app layout with header, sidebar, content area"
- "Configure React Router with route definitions"
- "Set up Tailwind CSS with design tokens"

---

## Layer 6: UI Features

User-facing pages, forms, and feature-specific interface components.

**Keywords:**
- page, view, screen, dialog, modal, drawer, panel
- form, input, field, select, checkbox, radio, textarea
- button, card, table, list, grid, chart, widget
- upload, download, file picker, drag and drop
- profile, dashboard, settings, admin, management
- search bar, filter panel, sort controls
- comment, like, share, follow, notification bell
- calendar, date picker, time picker, scheduler

**Patterns:**
- "Create X page/view", "Add X form", "Build X component"
- Tasks that build visible, interactive UI elements
- Tasks that implement specific user workflows
- Tasks that compose lower-level components into features

**Examples:**
- "Create user profile page"
- "Build registration form with validation"
- "Add dashboard with analytics charts"
- "Create file upload component"
- "Build notification center panel"

---

## Layer 7: Polish

Error handling, accessibility, loading states, final testing, and documentation.

**Keywords:**
- error, error handling, error boundary, fallback, retry, recovery
- loading, spinner, skeleton, progress, placeholder, suspense
- accessibility, a11y, aria, keyboard, screen reader, contrast
- empty state, no data, 404, not found
- toast, snackbar, notification (UI feedback), alert (UI feedback)
- animation, transition, micro-interaction
- responsive, mobile, breakpoint, adaptive
- test, testing, e2e, integration test, snapshot
- docs, documentation, readme, guide, help, tooltip
- performance, optimize, lazy, code split, bundle, cache
- seo, meta, og tag, structured data
- analytics (frontend), tracking (frontend)

**Patterns:**
- "Add error handling for X", "Add loading states to X"
- Tasks that improve existing features rather than adding new ones
- Tasks that add polish to previously built UI
- Tasks that write tests or documentation

**Examples:**
- "Add error handling and fallback UI for all pages"
- "Implement loading skeletons for data-fetching components"
- "Add ARIA labels and keyboard navigation"
- "Write E2E tests for registration flow"
- "Add page transitions and micro-interactions"

---

## Conflict Resolution

When a task matches multiple layers:

1. **Lowest layer wins.** If a task creates both a model (layer 1) and an API endpoint
   (layer 3), classify as layer 1. Foundation comes first.

2. **Split the task.** If a task genuinely spans two distant layers (e.g., layer 0 and
   layer 6), suggest splitting it into two tasks. Ask the user.

3. **When in doubt, ask.** If classification is genuinely ambiguous (could be layer 3
   or layer 4, for example), present both options to the user with rationale.

---

## Branch Assignment

Tasks at the same layer that have no dependency between them can run in parallel.
Assign them to separate branches:

1. **Default branch: "main"** — the critical path through the system.
2. **Named branches** — for independent features that can be built separately.

Rules:
- A task depends on everything in lower layers (implicit dependency).
- A task depends on its parent component's tasks (explicit dependency).
- Two tasks at the same layer with no shared component and no cross-references
  are independent → separate branches.
- If unsure whether two tasks are independent, keep them on the same branch
  (conservative — avoids ordering mistakes).

Branch naming: use the feature or component name (e.g., "auth", "notifications",
"reporting"). Keep names short and descriptive.
