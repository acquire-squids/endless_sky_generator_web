const log = document.getElementById("log");

export const println = (text) => {
  if (log) {
    log.innerText += text + "\n";
  } else {
    console.log(text);
  }
};

export const readFileAsText = async (file) => {
  return await new Promise((resolve) => {
    const reader = new FileReader();

    reader.onload = () => {
      resolve(reader.result);
    };

    reader.readAsText(file);
  });
};
