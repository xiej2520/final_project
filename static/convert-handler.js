const form = document.getElementById("convert-form");
form.onsubmit = (event) => {
  event.preventDefault();
  const formData = new FormData(form);
  let data = Object.fromEntries(formData);
  data.lat = Number(data.lat);
  data.long = Number(data.long);
  data.zoom = Number(data.zoom);
  console.log(data);
  fetch(form.action, {
    method: 'POST',
    headers: new Headers({ 'content-type': 'application/json' }),
    body: JSON.stringify(data)
  })
  .then(async response => {
    const json = await response.json();
    console.log(json);
    let t = JSON.stringify(json);
    let row = `<tr><td>${t}<br/>zoom=${data.zoom}</td><td><img src="/tiles/${data.zoom}/${json.x_tile}/${json.y_tile}.png" width="256" height="256"/></td></tr>`;
    document.getElementById("convert-responses").innerHTML += row;
  }
  )
  .catch(error => {
    console.log(error);
    document.getElementById("convert-responses").innerHTML +=
      `<tr><td>ERROR: ${JSON.stringify(error)}</td></tr>`
  })
};

document.getElementById("resetConvertButton").addEventListener("click", (event) => {
  event.preventDefault();
  document.getElementById("convert-responses").innerHTML = "";
});
