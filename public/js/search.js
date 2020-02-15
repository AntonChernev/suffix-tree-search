function search() {
    const part = document.getElementById('input').value;
    fetch(`/api/search?part=${encodeURIComponent(part)}`)
        .then(res => res.json())
        .then(data => {
            const searchResultsNode = document.getElementById('search-results');
            while(searchResultsNode.firstChild) {
                searchResultsNode.removeChild(searchResultsNode.firstChild);
            }
            data.forEach(text => {
                const childNode = document.createElement('li');
                childNode.innerHTML = text;
                searchResultsNode.appendChild(childNode);
            })
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
