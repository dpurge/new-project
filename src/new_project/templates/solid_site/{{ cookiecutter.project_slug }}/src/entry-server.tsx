import { StartServer, createHandler } from "@solidjs/start/server";
import { renderAsync } from "solid-js/web";

export default createHandler(() => <StartServer />, renderAsync);
