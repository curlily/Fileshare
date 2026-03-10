import icon_map from './file-icons.js';

function get_file_icon(ext) {
    if (!ext) return "📄";
    return icon_map[ext.toLowerCase()] ?? "📄";
}

export async function load_directory() {
    const path = window.location.pathname;

    //document.getElementById("path").textContent = path.replaceAll("/", " / ").trim()
    build_path_nav(path);

    const route = `/api/files${path}`.replace(/\/$/, ""); // Strip trailing slash
    const res = await fetch(route);
    const entries = await res.json();

    const table = document.getElementById("file-table-body");
    table.innerHTML = "";

    if(entries.length === 0) {
        table.append(get_placeholder_row());
        return;
    }

    for (const entry of entries) {
        table.append(get_entry_row(entry));
    }

    function build_path_nav(path) {

        const path_div = document.getElementById("path");
        const path_parts = path.split("/");

        path_parts.shift();
        path_div.innerHTML = ""

        if(path !== "/") {
            path_div.append(get_path_part("..", "/"));
        }

        for (const path_part of path_parts) {

            const index = path_parts.indexOf(path_part);
            const target = path_parts.slice(0, index + 1).join("/");

            path_div.append(" / ");
            path_div.append(get_path_part(path_part, target));
        }
    }

    function get_path_part(name, target) {

        const a = document.createElement("a");

        a.innerText = name
        a.onclick = () => {
            history.pushState({}, "", target);
            load_directory();
        }

        return a;
    }

    function get_entry_row(entry) {
        const tr = document.createElement("tr");
        const ext = entry.name.split('.').pop().toLowerCase();

        const icon = entry.is_dir ? "📁" : get_file_icon(ext);
        const lock = entry.requires_password ? ' 🔒' : '';

        tr.innerHTML = `
        <td>${icon}${lock} ${entry.name}</a></td>
        <td>${system_time_to_date_string(entry.created)}</td>
        <td>${system_time_to_date_string(entry.modified)}</td>
        <td>${entry.is_dir ? "" : format_bytes(entry.size)}</td>
        `;

        tr.style.cursor = "pointer";

        tr.onclick = async () => {

            if (entry.is_dir) {
                const next = `${path}/${entry.name}`.replace("//", "/");
                history.pushState({}, "", next);
                await load_directory();
            } else if (entry.requires_password) {
                await download_protected_file(path, entry);
            } else {
                await download_file(path, entry);
            }
        };

        return tr;
    }

    function get_placeholder_row() {
        const tr = document.createElement("tr");
        const td = document.createElement("td");

        td.innerText = "Empty directory";
        td.setAttribute("colspan", "4");
        tr.classList.add("placeholder_row");
        tr.append(td);

        return tr;
    }
}

async function download_protected_file(path, entry) {

    const root = document.getElementById("root");

    const overlay = document.getElementById("overlay");
    overlay.classList.add("dim");

    const wrapper = document.getElementById("password-wrapper");
    wrapper.classList.remove("hidden");

    const input = document.getElementById("password-input");
    const label = wrapper.querySelector("label");

    return new Promise((resolve) => {

        root.classList.add("no-interact");
        label.innerText = "Password for " + entry.name;
        input.focus()

        let last_input = input.value;

        input.onkeyup = (async (event) => {

            input.classList.remove('error');

            if (event.key === "Enter") {
                const password = input.value;

                let success = await download_file(path, entry, password);
                if (success) {
                    close_password_prompt()
                    resolve();
                }
            }

            if (event.key === 'Backspace' && last_input === "") {
                close_password_prompt()
                resolve();
            }

            last_input = input.value;
        });

        document.onkeyup = (event) => {
            if (event.key === "Escape") {
                close_password_prompt()
                resolve();
            }
        }
    });
}

function close_password_prompt() {

    const root = document.getElementById("root");
    const overlay = document.getElementById("overlay");
    const wrapper = document.getElementById("password-wrapper");
    const input = document.getElementById("password-input");

    overlay.classList.remove("dim");
    wrapper.classList.add("hidden");
    root.classList.remove("no-interact");
    input.value = "";
}

async function download_file(path, entry, password) {

    let route = `/api/files/${path}/${entry.name}`;

    if(password) {
        route += `?password=${encodeURIComponent(password)}`;
    }

    const res = await fetch(route, { method: "HEAD" });

    if (!res.ok) {
        if (res.status === 401 && entry.requires_password) {
            show_password_error()
        } else {
            console.error(`Download failed (${res.status})`);
        }
        return false;
    }

    const a = document.createElement('a');
    a.href = route;
    a.download = entry.name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);

    return true;
}

function show_password_error() {

    const input = document.getElementById("password-input");

    input.classList.add('error', 'shake');

    // Remove shake so it can be retriggered
    input.addEventListener(
        'animationend',
        () => input.classList.remove('shake'),
        { once: true }
    );
}

function format_bytes(bytes) {
    if (bytes === 0) return "0 B";
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(2) + " " + sizes[i];
}

function system_time_to_date_string(st) {
    const millis =
        st.secs_since_epoch * 1000 +
        Math.floor(st.nanos_since_epoch / 1_000_000);

    return new Date(millis).toLocaleString();
}