/* raudo — interacción del sitio.
   Sin dependencias: hash routing, lente del selector, reproductor y portapapeles. */

(function () {
  'use strict';

  var $ = function (sel, root) {
    return (root || document).querySelector(sel);
  };
  var $$ = function (sel, root) {
    return Array.prototype.slice.call((root || document).querySelectorAll(sel));
  };

  var VIEWS = ['inicio', 'que-es', 'descargar', 'verificar', 'novedades', 'versiones'];
  var TITLES = {
    inicio: 'Inicio',
    'que-es': 'Qué es',
    descargar: 'Descargar',
    verificar: 'Verificar',
    novedades: 'Novedades',
    versiones: 'Versiones',
  };

  var reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  var statusEl = $('#status');
  var statusTimer = 0;
  function announce(message) {
    if (!statusEl) return;
    statusEl.textContent = message;
    window.clearTimeout(statusTimer);
    statusTimer = window.setTimeout(function () {
      statusEl.textContent = '';
    }, 2400);
  }

  /* ------------------------------------------------------------ vistas */

  var nav = $('#nav');
  var lens = nav ? $('.seg-lens', nav) : null;
  var thumb = nav ? $('.seg-thumb', nav) : null;
  var stretchTimer = 0;
  var segTabs = $$('.seg-tab');
  var mobileButtons = $$('#mobile-nav button');
  var panels = {};
  VIEWS.forEach(function (id) {
    panels[id] = $('#panel-' + id);
  });

  function measure(tab) {
    return { x: tab.offsetLeft, w: tab.offsetWidth };
  }

  function setLensTo(tab, animate) {
    if (!nav || !tab) return;
    var next = measure(tab);
    var prevX = parseFloat(nav.style.getPropertyValue('--seg-x'));
    if (isNaN(prevX)) prevX = next.x;
    var distance = Math.abs(next.x - prevX);

    if (animate && distance > 2) {
      var stretch = Math.min(1.16, 1 + distance / 640);
      nav.style.setProperty('--seg-stretch', stretch.toFixed(3));
      nav.setAttribute('data-moving', '');
      window.clearTimeout(stretchTimer);
      stretchTimer = window.setTimeout(function () {
        nav.removeAttribute('data-moving');
        nav.style.setProperty('--seg-stretch', '1');
      }, 320);
    } else if (!animate) {
      nav.removeAttribute('data-moving');
      nav.style.setProperty('--seg-stretch', '1');
    }

    nav.style.setProperty('--seg-x', next.x + 'px');
    nav.style.setProperty('--seg-w', next.w + 'px');
  }

  function moveLens() {
    var active = nav ? $('.seg-tab.is-active', nav) : null;
    setLensTo(active, false);
  }

  if (nav) {
    var liftTo = function (tab) {
      if (!nav || !tab) return;
      nav.setAttribute('data-lifted', '');
      setLensTo(tab, true);
    };

    nav.addEventListener('pointerover', function (event) {
      var tab = event.target.closest('.seg-tab');
      if (tab) liftTo(tab);
    });

    nav.addEventListener('pointermove', function (event) {
      var rect = nav.getBoundingClientRect();
      nav.style.setProperty('--mx', event.clientX - rect.left + 'px');
    });

    nav.addEventListener('pointerleave', function () {
      nav.removeAttribute('data-lifted');
      window.clearTimeout(stretchTimer);
      nav.removeAttribute('data-moving');
      nav.style.setProperty('--seg-stretch', '1');
      var active = $('.seg-tab.is-active', nav);
      if (active) setLensTo(active, false);
    });

    nav.addEventListener('focusin', function (event) {
      var tab = event.target.closest('.seg-tab');
      if (tab) liftTo(tab);
    });

    nav.addEventListener('focusout', function (event) {
      if (nav.contains(event.relatedTarget)) return;
      nav.removeAttribute('data-lifted');
      var active = $('.seg-tab.is-active', nav);
      if (active) setLensTo(active, false);
    });
  }

  function currentView() {
    var hash = window.location.hash.replace('#', '');
    return VIEWS.indexOf(hash) >= 0 ? hash : 'inicio';
  }

  function render(view, focusPanel) {
    VIEWS.forEach(function (id) {
      if (panels[id]) panels[id].hidden = id !== view;
    });
    segTabs.forEach(function (tab) {
      var on = tab.dataset.view === view;
      tab.classList.toggle('is-active', on);
      tab.setAttribute('aria-selected', on ? 'true' : 'false');
      tab.tabIndex = on ? 0 : -1;
    });
    mobileButtons.forEach(function (btn) {
      btn.classList.toggle('is-active', btn.dataset.view === view);
    });
    var title = $('#window-title');
    if (title) title.textContent = TITLES[view] || 'Inicio';
    var section = $('.window');
    if (section) section.setAttribute('aria-label', TITLES[view] || 'Inicio');
    moveLens();
    window.scrollTo({ top: 0, behavior: reduceMotion ? 'auto' : 'smooth' });
    if (focusPanel && panels[view]) {
      var heading = $('h1, h2', panels[view]);
      if (heading) {
        heading.setAttribute('tabindex', '-1');
        heading.focus({ preventScroll: true });
      }
    }
  }

  function go(view, push) {
    if (VIEWS.indexOf(view) < 0) view = 'inicio';
    if (push !== false && window.location.hash !== '#' + view) {
      window.history.pushState(null, '', '#' + view);
    }
    render(view, push !== false);
  }

  segTabs.forEach(function (tab) {
    tab.addEventListener('click', function () {
      go(tab.dataset.view);
      closeMenu();
    });
  });

  mobileButtons.forEach(function (btn) {
    btn.addEventListener('click', function () {
      go(btn.dataset.view);
      closeMenu();
    });
  });

  $$('[data-view]').forEach(function (el) {
    if (el.classList.contains('seg-tab') || el.closest('#mobile-nav')) return;
    el.addEventListener('click', function (event) {
      event.preventDefault();
      go(el.dataset.view);
    });
  });

  window.addEventListener('hashchange', function () {
    render(currentView(), false);
  });
  window.addEventListener('popstate', function () {
    render(currentView(), false);
  });
  window.addEventListener('resize', moveLens);

  /* ------------------------------------------------------------ menú móvil */

  var menuBtn = $('#menu-btn');
  var mobileNav = $('#mobile-nav');

  function closeMenu() {
    if (!menuBtn || !mobileNav) return;
    mobileNav.classList.remove('is-open');
    menuBtn.setAttribute('aria-expanded', 'false');
    menuBtn.setAttribute('aria-label', 'Abrir menú');
    menuBtn.innerHTML = '<svg class="ico" aria-hidden="true"><use href="#i-menu" /></svg>';
  }

  if (menuBtn && mobileNav) {
    menuBtn.addEventListener('click', function () {
      var open = !mobileNav.classList.contains('is-open');
      mobileNav.classList.toggle('is-open', open);
      menuBtn.setAttribute('aria-expanded', open ? 'true' : 'false');
      menuBtn.setAttribute('aria-label', open ? 'Cerrar menú' : 'Abrir menú');
      menuBtn.innerHTML =
        '<svg class="ico" aria-hidden="true"><use href="#' + (open ? 'i-close' : 'i-menu') + '" /></svg>';
    });
    document.addEventListener('keydown', function (event) {
      if (event.key === 'Escape') closeMenu();
    });
  }

  /* ------------------------------------------------------------ cabecera */

  var header = $('#site-header');
  function onScroll() {
    if (header) header.classList.toggle('is-scrolled', window.scrollY > 16);
  }
  onScroll();
  window.addEventListener('scroll', onScroll, { passive: true });

  /* ------------------------------------------------------------ copiar */

  function legacyCopy(text) {
    var area = document.createElement('textarea');
    area.value = text;
    area.setAttribute('readonly', '');
    area.style.position = 'fixed';
    area.style.top = '-1000px';
    document.body.appendChild(area);
    area.select();
    var ok = false;
    try {
      ok = document.execCommand('copy');
    } catch (error) {
      ok = false;
    }
    document.body.removeChild(area);
    return ok;
  }

  function copyText(text) {
    if (navigator.clipboard && window.isSecureContext) {
      return navigator.clipboard.writeText(text).then(
        function () {
          return true;
        },
        function () {
          return legacyCopy(text);
        },
      );
    }
    return Promise.resolve(legacyCopy(text));
  }

  $$('[data-copy]').forEach(function (button) {
    var label = $('span', button);
    var original = label ? label.textContent : '';
    button.addEventListener('click', function () {
      copyText(button.dataset.copy).then(function (ok) {
        if (!ok) {
          announce('No se pudo copiar el texto');
          return;
        }
        announce('Copiado al portapapeles');
        if (!label) return;
        label.textContent = button.dataset.copied || 'Copiado';
        window.setTimeout(function () {
          label.textContent = original;
        }, 1600);
      });
    });
  });

  /* ------------------------------------------------------------ reproductor */

  var player = $('#player');
  var video = $('#video');

  if (player && video) {
    var big = $('#player-big');
    var toggle = $('#player-toggle');
    var timeEl = $('#player-time');
    var range = $('#player-range');
    var muteBtn = $('#player-mute');
    var fullBtn = $('#player-full');
    var duration = 19;

    function fmt(value) {
      var total = Math.max(0, Math.floor(value || 0));
      var minutes = Math.floor(total / 60);
      var seconds = total % 60;
      return minutes + ':' + String(seconds).padStart(2, '0');
    }

    function paint() {
      var d = duration || 19;
      var t = Math.min(video.currentTime || 0, d);
      if (timeEl) timeEl.textContent = fmt(t) + ' / ' + fmt(d);
      if (range) {
        range.max = String(d);
        range.value = String(t);
        range.style.setProperty('--p', ((t / d) * 100).toFixed(2) + '%');
      }
    }

    function play() {
      if (video.ended) video.currentTime = 0;
      var result = video.play();
      if (result && result.catch) result.catch(function () {});
    }

    function flip() {
      if (video.paused) play();
      else video.pause();
    }

    function setIcon(button, id) {
      if (!button) return;
      button.innerHTML = '<svg class="ico" aria-hidden="true"><use href="#' + id + '" /></svg>';
    }

    video.addEventListener('loadedmetadata', function () {
      duration = video.duration && isFinite(video.duration) ? video.duration : 19;
      paint();
    });
    video.addEventListener('timeupdate', paint);
    video.addEventListener('play', function () {
      player.classList.add('is-playing');
      player.classList.remove('is-ended');
      setIcon(toggle, 'i-pause');
      if (toggle) toggle.setAttribute('aria-label', 'Pausar');
      if (big) big.setAttribute('aria-label', 'Pausar');
    });
    video.addEventListener('pause', function () {
      player.classList.remove('is-playing');
      setIcon(toggle, 'i-play');
      if (toggle) toggle.setAttribute('aria-label', 'Reproducir');
      if (big) big.setAttribute('aria-label', 'Reproducir');
    });
    video.addEventListener('ended', function () {
      player.classList.remove('is-playing');
      player.classList.add('is-ended');
      setIcon(big, 'i-replay');
      if (big) big.setAttribute('aria-label', 'Volver a ver');
    });

    if (toggle) toggle.addEventListener('click', flip);
    if (big) big.addEventListener('click', flip);
    video.addEventListener('click', flip);

    if (range) {
      range.addEventListener('input', function () {
        video.currentTime = Number(range.value);
        paint();
      });
    }

    if (muteBtn) {
      muteBtn.addEventListener('click', function () {
        video.muted = !video.muted;
        muteBtn.setAttribute('aria-pressed', video.muted ? 'true' : 'false');
        muteBtn.setAttribute('aria-label', video.muted ? 'Activar el sonido' : 'Silenciar');
        setIcon(muteBtn, video.muted ? 'i-mute' : 'i-volume');
      });
    }

    if (fullBtn) {
      fullBtn.addEventListener('click', function () {
        if (document.fullscreenElement) {
          document.exitFullscreen();
        } else if (player.requestFullscreen) {
          var result = player.requestFullscreen();
          if (result && result.catch) result.catch(function () {});
        }
      });
    }

    player.addEventListener('keydown', function (event) {
      if (event.target !== event.currentTarget) return;
      var key = event.key.toLowerCase();
      if (event.key === ' ' || event.key === 'Enter' || key === 'k') {
        event.preventDefault();
        flip();
        return;
      }
      if (event.key === 'ArrowLeft') {
        event.preventDefault();
        video.currentTime = Math.max(0, video.currentTime - 5);
        return;
      }
      if (event.key === 'ArrowRight') {
        event.preventDefault();
        video.currentTime = Math.min(duration, video.currentTime + 5);
        return;
      }
      if (key === 'm' && muteBtn) {
        event.preventDefault();
        muteBtn.click();
        return;
      }
      if (key === 'f' && fullBtn) {
        event.preventDefault();
        fullBtn.click();
      }
    });

    paint();
  }

  /* ------------------------------------------------------------ versiones */

  var search = $('#version-search');
  if (search) {
    var rows = $$('#versions-body tr');
    var empty = $('#versions-empty');
    var tableWrap = $('#versions-table-wrap');

    search.addEventListener('input', function () {
      var query = search.value.trim().toLowerCase();
      var shown = 0;
      rows.forEach(function (row) {
        var haystack = (row.dataset.search || row.textContent || '').toLowerCase();
        var match = !query || haystack.indexOf(query) >= 0;
        row.hidden = !match;
        if (match) shown += 1;
      });
      if (empty) empty.hidden = shown !== 0;
      if (tableWrap) tableWrap.hidden = shown === 0;
    });
  }

  /* ------------------------------------------------------------ brillo de cristal */

  $$('.btn').forEach(function (el) {
    el.addEventListener('pointermove', function (event) {
      var rect = el.getBoundingClientRect();
      el.style.setProperty('--lx', event.clientX - rect.left + 'px');
      el.style.setProperty('--ly', event.clientY - rect.top + 'px');
    });
  });

  /* ------------------------------------------------------------ arranque */

  render(currentView(), false);
  window.requestAnimationFrame(moveLens);
  window.addEventListener('load', moveLens);
  if (document.fonts && document.fonts.ready) {
    document.fonts.ready.then(moveLens);
  }
  document.documentElement.lang = 'es';
})();
