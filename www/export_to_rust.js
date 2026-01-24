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
