export const readFileAsText = async (file) => {
  return await new Promise((resolve) => {
    const reader = new FileReader();

    reader.onload = () => {
      resolve(reader.result);
    };

    reader.readAsText(file);
  });
};
