let currentSession = null;

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    loadHistory();
});

async function startScraping() {
    const urlsText = document.getElementById('urls').value.trim();

    if (!urlsText) {
        showError('Please enter at least one URL');
        return;
    }

    const urls = urlsText.split('\n').map(u => u.trim()).filter(u => u);
    const enablePagination = document.getElementById('enablePagination').checked;
    const maxPages = parseInt(document.getElementById('maxPages').value) || 0;
    const rateLimit = parseFloat(document.getElementById('rateLimit').value) || 2.0;

    let customSelectors = null;
    const customSelectorsText = document.getElementById('customSelectors').value.trim();

    if (customSelectorsText) {
        try {
            customSelectors = JSON.parse(customSelectorsText);
        } catch (e) {
            showError('Invalid JSON in custom selectors');
            return;
        }
    }

    const request = {
        urls,
        enable_pagination: enablePagination,
        max_pages: maxPages,
        rate_limit: rateLimit,
        custom_selectors: customSelectors
    };

    // Update UI
    setLoading(true);
    hideResults();

    try {
        const response = await fetch('/api/scrape', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request)
        });

        const data = await response.json();

        if (data.success && data.session) {
            currentSession = data.session;
            displayResults(data.session);
            showSuccess(data.message);
            await loadHistory();
        } else {
            showError(data.message || 'Scraping failed');
        }
    } catch (error) {
        showError(`Error: ${error.message}`);
    } finally {
        setLoading(false);
    }
}

function displayResults(session) {
    const resultsSection = document.getElementById('resultsSection');
    resultsSection.style.display = 'block';

    // Display stats
    const statsHtml = `
        <div class="stat-card">
            <div class="stat-value">${session.total_pages_scraped}</div>
            <div class="stat-label">Pages Scraped</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">${session.total_links_found}</div>
            <div class="stat-label">Links Found</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">${session.total_images_found}</div>
            <div class="stat-label">Images Found</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">${session.errors.length}</div>
            <div class="stat-label">Errors</div>
        </div>
    `;
    document.getElementById('stats').innerHTML = statsHtml;

    // Display results
    const resultsHtml = session.results.map(result => `
        <div class="result-item">
            <h3>${result.content.title || 'Untitled Page'}</h3>
            <div class="result-url">
                <strong>URL:</strong> <a href="${result.url}" target="_blank">${result.url}</a>
                <br>
                <strong>Page:</strong> ${result.page_number} |
                <strong>Timestamp:</strong> ${new Date(result.timestamp).toLocaleString()}
            </div>

            ${result.content.content.length > 0 ? `
                <div class="content-section">
                    <h4>📄 Content (${result.content.content.length} paragraphs)</h4>
                    <ul class="content-list">
                        ${result.content.content.slice(0, 3).map(text =>
                            `<li>${truncate(text, 200)}</li>`
                        ).join('')}
                        ${result.content.content.length > 3 ?
                            `<li><em>... and ${result.content.content.length - 3} more</em></li>`
                            : ''}
                    </ul>
                </div>
            ` : ''}

            ${result.content.links.length > 0 ? `
                <div class="content-section">
                    <h4>🔗 Links (${result.content.links.length})</h4>
                    <ul class="content-list">
                        ${result.content.links.slice(0, 5).map(link => `
                            <li class="link-item">
                                <span>
                                    ${link.text || 'No text'}
                                    ${link.is_external ? '<span class="external-badge">External</span>' : ''}
                                </span>
                                <a href="${link.href}" target="_blank" title="${link.href}">
                                    ${truncate(link.href, 50)}
                                </a>
                            </li>
                        `).join('')}
                        ${result.content.links.length > 5 ?
                            `<li><em>... and ${result.content.links.length - 5} more</em></li>`
                            : ''}
                    </ul>
                </div>
            ` : ''}

            ${result.content.images.length > 0 ? `
                <div class="content-section">
                    <h4>🖼️ Images (${result.content.images.length})</h4>
                    <ul class="content-list">
                        ${result.content.images.slice(0, 3).map(img => `
                            <li>
                                ${img.alt || 'No alt text'} -
                                <a href="${img.src}" target="_blank">${truncate(img.src, 60)}</a>
                            </li>
                        `).join('')}
                        ${result.content.images.length > 3 ?
                            `<li><em>... and ${result.content.images.length - 3} more</em></li>`
                            : ''}
                    </ul>
                </div>
            ` : ''}

            ${Object.keys(result.content.metadata).length > 0 ? `
                <div class="content-section">
                    <h4>📋 Metadata</h4>
                    <ul class="content-list">
                        ${Object.entries(result.content.metadata).map(([key, value]) => `
                            <li><strong>${key}:</strong> ${truncate(value, 150)}</li>
                        `).join('')}
                    </ul>
                </div>
            ` : ''}
        </div>
    `).join('');

    document.getElementById('resultsContent').innerHTML = resultsHtml;

    // Scroll to results
    resultsSection.scrollIntoView({ behavior: 'smooth' });
}

async function loadHistory() {
    try {
        const response = await fetch('/api/sessions');
        const sessions = await response.json();

        const historyList = document.getElementById('historyList');

        if (sessions.length === 0) {
            historyList.innerHTML = '<p class="empty-state">No scraping sessions yet. Start scraping to see history.</p>';
            return;
        }

        const historyHtml = sessions.map((session, index) => `
            <div class="history-item" onclick="loadSession(${index})">
                <div class="history-header">
                    <strong>${session.config.urls[0]} ${session.config.urls.length > 1 ? `(+${session.config.urls.length - 1} more)` : ''}</strong>
                    <span class="history-time">${new Date(session.start_time).toLocaleString()}</span>
                </div>
                <div class="history-stats">
                    <span>📄 ${session.total_pages_scraped} pages</span>
                    <span>🔗 ${session.total_links_found} links</span>
                    <span>🖼️ ${session.total_images_found} images</span>
                    ${session.errors.length > 0 ? `<span>⚠️ ${session.errors.length} errors</span>` : ''}
                </div>
            </div>
        `).join('');

        historyList.innerHTML = historyHtml;
    } catch (error) {
        console.error('Failed to load history:', error);
    }
}

async function loadSession(index) {
    try {
        const response = await fetch(`/api/sessions/${index}`);
        const session = await response.json();

        if (session) {
            currentSession = session;
            displayResults(session);
            document.getElementById('resultsSection').scrollIntoView({ behavior: 'smooth' });
        }
    } catch (error) {
        showError('Failed to load session');
    }
}

function downloadResults() {
    if (!currentSession) {
        showError('No results to download');
        return;
    }

    const dataStr = JSON.stringify(currentSession, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);

    const link = document.createElement('a');
    link.href = url;
    link.download = `scraping-results-${new Date().getTime()}.json`;
    link.click();

    URL.revokeObjectURL(url);
}

async function clearResults() {
    if (!confirm('Clear all scraping history?')) {
        return;
    }

    try {
        await fetch('/api/sessions', { method: 'DELETE' });
        currentSession = null;
        hideResults();
        await loadHistory();
        showSuccess('History cleared');
    } catch (error) {
        showError('Failed to clear history');
    }
}

function setLoading(loading) {
    const btn = document.getElementById('scrapeBtn');
    const btnText = document.getElementById('btnText');
    const btnSpinner = document.getElementById('btnSpinner');

    btn.disabled = loading;
    btnText.style.display = loading ? 'none' : 'inline';
    btnSpinner.style.display = loading ? 'inline-block' : 'none';
}

function showSuccess(message) {
    const statusMessage = document.getElementById('statusMessage');
    statusMessage.className = 'status-message success';
    statusMessage.textContent = `✓ ${message}`;
    statusMessage.style.display = 'block';
}

function showError(message) {
    const statusMessage = document.getElementById('statusMessage');
    statusMessage.className = 'status-message error';
    statusMessage.textContent = `✗ ${message}`;
    statusMessage.style.display = 'block';

    const resultsSection = document.getElementById('resultsSection');
    resultsSection.style.display = 'block';
    resultsSection.scrollIntoView({ behavior: 'smooth' });
}

function hideResults() {
    document.getElementById('resultsSection').style.display = 'none';
}

function truncate(str, length) {
    if (str.length <= length) return str;
    return str.substring(0, length) + '...';
}
