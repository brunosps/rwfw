// Boots Turbo + Stimulus for every page. Loaded from base.html.j2 as an
// external module so the CSP needs no 'unsafe-inline' for scripts.
//
// Imports use the served paths (/vendor/..., /assets/...) instead of bare
// specifiers, so no inline import map is needed either. App controllers do the
// same: `import { Controller } from "/vendor/stimulus.min.js"`.
import "/vendor/turbo.min.js"
import { Application } from "/vendor/stimulus.min.js"
import ReactiveController from "/assets/reactive.js"

window.Stimulus = Application.start()
window.Stimulus.register("reactive", ReactiveController)
