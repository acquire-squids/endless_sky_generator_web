export const readFileAsText = async (file) => {
  return await new Promise((resolve) => {
    const reader = new FileReader();

    reader.onload = () => {
      resolve(reader.result);
    };

    reader.readAsText(file);
  });
};

const input = document.getElementById("input");

let defaults_checked = false;

const default_paths = [];
const default_sources = [];

const loaded_paths = [];
const loaded_sources = [];

input.addEventListener("change", async () => {
  for (const file of event.target.files) {
    if (file.type.startsWith("text")) {
      loaded_paths.push(file.name);
      loaded_sources.push(await readFileAsText(file));
    }
  }
});

const include_defaults = document.getElementById("include-defaults");
const clear_uploads = document.getElementById("clear-uploads");

clear_uploads.addEventListener("click", () => {
  loaded_paths.splice(0, loaded_paths.length);
  loaded_sources.splice(0, loaded_sources.length);
});

export const getPathsAndSources = async () => {
  if (!defaults_checked && include_defaults.checked) {
    const es_stable_data_paths = await fetch(new Request("es_stable_data_paths.txt"))
      .then(async (response) => {
        if (!response.ok) {
          // poor man's `Option<T>`: an empty array
          return [];
        } else {
          return [await response.text()];
        }
      });

    if (es_stable_data_paths.length >= 1) {
      for (const path of es_stable_data_paths[0].split("\n")) {
        let sources = await fetch(path)
          .then(async (response) => {
            if (!response.ok) {
              return [];
            } else {
              return [await response.text()];
            }
          });

        if (sources.length >= 1) {
          default_paths.push(path);
          default_sources.push(sources[0]);
        }
      }

      defaults_checked = true;
    }
  }

  return {
    paths: (include_defaults.checked ? default_paths.concat(loaded_paths) : loaded_paths),
    sources: (include_defaults.checked ? default_sources.concat(loaded_sources) : loaded_sources),
  };
};

export const downloadZip = (fileName, bytes) => {
  const zipBlob = new Blob(
    [bytes.buffer],
    {
      type: "application/zip"
    }
  );

  const blobUrl = URL.createObjectURL(zipBlob);

  const downloadLink = document.createElement("a");

  downloadLink.setAttribute("href", blobUrl);
  downloadLink.setAttribute("download", fileName);
  downloadLink.click();

  downloadLink.remove();

  URL.revokeObjectURL(blobUrl);
};

export const generateAndDownload = (fileName, rustFn) => {
  return async () => {
    const paths_and_sources = await getPathsAndSources();

    let result;

    try {
      result = new Uint8Array(rustFn(paths_and_sources.paths, paths_and_sources.sources));
    } catch(error) {
      console.error(error);
      return;
    }

    downloadZip(fileName, result);
  }
};

export const iterateElements = (node, modify) => {
  const elements = [node];

  while (elements.length > 0) {
    const it = elements.shift();

    if (it === undefined) {
      break;
    }

    modify(it);

    for (const child of it.children) {
      elements.push(child);
    }
  }
};

export const defaultEventListeners = (node) => {
  if (node.classList.contains("paired-range")) {
    node.addEventListener("input", () => {
      if (event.target.checkValidity()) {
        event.target.nextElementSibling.value = event.target.value;
      }
    });
  }

  if (node.classList.contains("paired-range-output")) {
    node.addEventListener("input", () => {
      if (event.target.checkValidity()) {
        event.target.previousElementSibling.value = event.target.value;
      }
    });
  }

  if (node.classList.contains("click-to-remove")) {
    node.addEventListener("click", (event) => {
      // I couldn't get normal "commandfor" and a "command" event to work
      const target = document.getElementById(event.target.getAttribute("data-commandfor"));

      if (target !== null) {
        const other_targets = Array.from(document.getElementsByClassName(target.className))
          .filter((other_target) => other_target.id !== target.id);

        // only remove if there's one left to clone
        if (other_targets.length > 0) {
          // there might be a button for cloning this target; point it to another valid target if so
          for (const potential_create_button of document.getElementsByClassName("click-to-create")) {
            if (potential_create_button.getAttribute("data-commandfor") == target.id) {
              potential_create_button.setAttribute("data-commandfor", other_targets[other_targets.length - 1].id);
            }
          }

          target.remove();
        }
      }
    });
  }

  if (node.classList.contains("click-to-create")) {
    node.addEventListener("click", (event) => {
      const target = document.getElementById(event.target.getAttribute("data-commandfor"));

      if (target !== null) {
        const clone = deepClone(target);

        event.target.before(clone);

        iterateElements(clone, defaultEventListeners);
      }
    });
  }
};

let clone_counter = 0;

export const deepClone = (node) => {
  const the_clone = node.cloneNode(true);

  const id_map = new Map();

  iterateElements(the_clone, (clone) => {
    const clone_id = clone.getAttribute("id");

    if (clone_id !== null) {
      const clone_original_id_end = clone_id.indexOf("__ONLY_THE_END_PLEASE_AND_THANK_YOU__");
      const clone_original_id = clone_original_id_end >= 0 ? clone_id.substring(0, clone_original_id_end) : clone_id;

      const next_id = clone_original_id + "__ONLY_THE_END_PLEASE_AND_THANK_YOU__" + clone_counter.toString();

      id_map.set(clone_id, next_id);

      clone.setAttribute("id", next_id);
    }

    const clone_name = clone.getAttribute("name");

    if (clone_name !== null) {
      const clone_original_name_end = clone_name.indexOf("__ONLY_THE_END_PLEASE_AND_THANK_YOU__");
      const clone_original_name = clone_original_name_end >= 0 ? clone_name.substring(0, clone_original_name_end) : clone_name;

      clone.setAttribute("name", clone_original_name + "__ONLY_THE_END_PLEASE_AND_THANK_YOU__" + clone_counter.toString());
    }

    const clone_commandfor = clone.getAttribute("data-commandfor");

    if (clone_commandfor !== null && id_map.get(clone_commandfor) !== undefined) {
      clone.setAttribute("data-commandfor", id_map.get(clone_commandfor));
    }

    clone_counter += 1;
  });

  return the_clone;
};
