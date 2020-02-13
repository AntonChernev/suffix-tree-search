function search() {
    const part = document.getElementById('input').value;
    fetch(`/api/search?part=${encodeURIComponent(part)}`)
        .then(res => res.text())
        .then(data => {
            document.getElementById('search-result').innerText = data;
        })
        .catch(err => {
            console.error(err);
        });
}

function getText() {
    fetch('/api/text')
        .then(res => res.text())
        .then(data => {
            document.getElementById('text').innerHTML = data;
        })
        .catch(err => {
            console.error(err);
        })
}