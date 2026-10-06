import { mount } from "svelte";
import App from "./App.svelte";
import "./styles/tokens.css";
import "./styles/markdown.css";

mount(App, {
  target: document.querySelector("#app"),
});
