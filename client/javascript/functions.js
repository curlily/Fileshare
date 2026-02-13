async function loadDirectory() {
    const path = window.location.pathname;

    //document.getElementById("path").textContent = path.replaceAll("/", " / ").trim()
    build_path_nav(path);

    const route = `/api/files${path}`.replace(/\/$/, ""); // Strip trailing slash
    const res = await fetch(route);
    const entries = await res.json();

    const table = document.getElementById("file-table-body");
    table.innerHTML = "";

    for (const entry of entries) {
        table.appendChild(get_entry_row(entry));
    }

    function build_path_nav(path) {

        const path_div = document.getElementById("path");
        const path_parts = path.split("/");

        path_parts.shift();
        path_div.innerHTML = ""

        for (const path_part of path_parts) {

            const a = document.createElement("a");
            a.innerText = path_part

            const index = path_parts.indexOf(path_part);
            const target = path_parts.slice(0, index + 1).join("/");

            a.onclick = () => {
                history.pushState({}, "", target);
                loadDirectory();
            }

            path_div.append(" / ");
            path_div.append(a);
        }
    }

    function get_entry_row(entry) {
        const tr = document.createElement("tr");
        const ext = entry.name.split('.').pop().toLowerCase();

        const icon = entry.is_dir ? "📁" : getFileIcon(ext);
        //const lock = protectedFlag ? ' 🔒' : '';

        tr.innerHTML = `
        <td>${icon} ${entry.name}</a></td>
        <td>${systemTimeToDateString(entry.created)}</td>
        <td>${systemTimeToDateString(entry.modified)}</td>
        <td>${entry.is_dir ? "-" : format_bytes(entry.size)}</td>
        `;

        tr.style.cursor = "pointer";

        tr.onclick = () => {

            if (entry.is_dir) {
                const next = `${path}/${entry.name}`.replace("//", "/");
                history.pushState({}, "", next);
                loadDirectory();
            } else {
                window.location.href = `api/files/${path}/${entry.name}`;
            }
        };

        return tr;
    }
}

function getFileIcon(ext) {
    switch (ext) {
        case "jpg":
        case "jpeg":
        case "png":
        case "gif":
            return "🖼️";
        case "mp4":
        case "mov":
        case "avi":
            return "🎞️";
        case "mp3":
        case "wav":
            return "🎵";
        case "pdf":
            return "📄";
        case "zip":
        case "rar":
            return "🗃️";
        case "txt":
        case "md":
        case "doc":
        case "docx":
            return "📝";
        default:
            return "📄";
    }
}

function format_bytes(bytes) {
    if (bytes === 0) return "0 B";
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(2) + " " + sizes[i];
}

function systemTimeToDateString(st) {
    const millis =
        st.secs_since_epoch * 1000 +
        Math.floor(st.nanos_since_epoch / 1_000_000);

    return new Date(millis).toLocaleString();
}