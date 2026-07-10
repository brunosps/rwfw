import * as Turbo from "@hotwired/turbo"
import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  connect() {
    this.queue = Promise.resolve()
  }

  trigger(event) {
    if (event && event.currentTarget !== window) {
      event.preventDefault()
    }

    const trigger = event.currentTarget
    const action = trigger.dataset.reactiveAction
    if (!action) {
      console.error("reactive trigger missing data-reactive-action")
      return
    }

    const explicit = this.explicitParams(trigger)
    const fields = this.namedFields()
    const params = { ...fields, ...explicit }

    this.queue = this.queue
      .then(() => this.post(action, params))
      .catch((error) => console.error("reactive action failed", error))
  }

  explicitParams(trigger) {
    const raw = trigger.dataset.reactiveParams
    if (!raw) return {}

    try {
      const parsed = JSON.parse(raw)
      return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {}
    } catch (error) {
      console.error("reactive params are not valid JSON", error)
      return {}
    }
  }

  namedFields() {
    const params = {}
    const fields = this.element.querySelectorAll("input[name], select[name], textarea[name]")

    for (const field of fields) {
      const root = field.closest('[data-controller~="reactive"]')
      if (root && root !== this.element) continue
      if (field.disabled) continue
      if (field.type === "radio" && !field.checked) continue

      params[field.name] = this.fieldValue(field)
    }

    return params
  }

  fieldValue(field) {
    if (field.type === "checkbox") {
      return field.checked ? (field.value === "on" ? true : field.value) : false
    }

    if (field.tagName === "SELECT" && field.multiple) {
      return Array.from(field.selectedOptions).map((option) => option.value)
    }

    return field.value
  }

  async post(action, params) {
    const token = this.element.dataset.reactiveToken
    const response = await fetch("/__rwfw/reactive/actions", {
      method: "POST",
      credentials: "same-origin",
      headers: this.headers(),
      body: JSON.stringify({ token, act: action, params }),
    })

    const body = await response.text()
    if (!response.ok) {
      console.error("reactive action failed", body)
      return
    }

    this.refreshTokenFromStream(body)
    if (body) {
      Turbo.renderStreamMessage(body)
    }
  }

  headers() {
    const headers = {
      "Accept": "text/vnd.turbo-stream.html",
      "Content-Type": "application/json",
    }
    const csrf = document.querySelector('meta[name="csrf-token"]')?.content
    if (csrf) headers["X-CSRF-Token"] = csrf
    return headers
  }

  refreshTokenFromStream(streamHtml) {
    const doc = new DOMParser().parseFromString(streamHtml, "text/html")
    const nextRoot = doc.querySelector('[data-controller~="reactive"][data-reactive-token]')
    if (nextRoot) {
      this.element.dataset.reactiveToken = nextRoot.dataset.reactiveToken
    }
  }
}
