// Catppuccin Mocha and Latte colors for the sketches, and the icon paths they use.
// The settings app reads its colors from whichever theme is active; these two
// flavors stand in for every dark and every light theme.

const FLAVORS = {
    mocha: {
        name: 'Mocha', dark: true,
        crust: '#11111B', mantle: '#181825', base: '#1E1E2E', surface0: '#313244', surface1: '#45475A', surface2: '#585B70',
        overlay0: '#6C7086', overlay1: '#7F849C', subtext0: '#A6ADC8', text: '#CDD6F4',
        accent: '#89B4FA', green: '#A6E3A1', yellow: '#F9E2AF', red: '#F38BA8', selected: '#89B4FA2E',
        accents: [['rosewater', '#F5E0DC'], ['flamingo', '#F2CDCD'], ['pink', '#F5C2E7'], ['mauve', '#CBA6F7'], ['red', '#F38BA8'], ['maroon', '#EBA0AC'], ['peach', '#FAB387'],
            ['yellow', '#F9E2AF'], ['green', '#A6E3A1'], ['teal', '#94E2D5'], ['sky', '#89DCEB'], ['sapphire', '#74C7EC'], ['blue', '#89B4FA'], ['lavender', '#B4BEFE']],
        bar: { background: '#1E1E2E', foreground: '#89B4FA', activeWorkspace: '#435375', activeWorkspaceText: '#CDD6F4', inactiveWorkspace: '#89B4FA', widgetBackground: '#313244', pillBorder: '#89B4FA' },
        desktop: '#2A2B3D',
    },
    latte: {
        name: 'Latte', dark: false,
        crust: '#DCE0E8', mantle: '#E6E9EF', base: '#EFF1F5', surface0: '#CCD0DA', surface1: '#BCC0CC', surface2: '#ACB0BE',
        overlay0: '#9CA0B0', overlay1: '#8C8FA1', subtext0: '#6C6F85', text: '#4C4F69',
        accent: '#1E66F5', green: '#40A02B', yellow: '#DF8E1D', red: '#D20F39', selected: '#1E66F524',
        accents: [['rosewater', '#DC8A78'], ['flamingo', '#DD7878'], ['pink', '#EA76CB'], ['mauve', '#8839EF'], ['red', '#D20F39'], ['maroon', '#E64553'], ['peach', '#FE640B'],
            ['yellow', '#DF8E1D'], ['green', '#40A02B'], ['teal', '#179299'], ['sky', '#04A5E5'], ['sapphire', '#209FB5'], ['blue', '#1E66F5'], ['lavender', '#7287FD']],
        bar: { background: '#EFF1F5', foreground: '#1E66F5', activeWorkspace: '#7287D5', activeWorkspaceText: '#EFF1F5', inactiveWorkspace: '#1E66F5', widgetBackground: '#CCD0DA', pillBorder: '#1E66F5' },
        desktop: '#C9CEDB',
    },
};

// ArtCraft roles: dark cards sit above the page, light cards are the lightest surface.
function roles(key) {
    const p = FLAVORS[key];
    if (!p) throw new Error(`unknown flavor ${key}; use ${Object.keys(FLAVORS).join(' or ')}`);
    return {
        ...p,
        page: p.dark ? p.base : p.mantle,
        card: p.dark ? p.surface0 : p.base,
        line: p.dark ? p.surface1 : p.surface0,
        control: p.dark ? p.base : p.mantle,
        tint: (hex, alpha) => hex + alpha,
    };
}

const ICONS = {
    general: 'M21 4H14 M10 4H3 M21 12H12 M8 12H3 M21 20H16 M12 20H3 M14 2V6 M8 10V14 M16 18V22',
    layout: 'M3 3H21V21H3Z M12 3V21 M12 12H21',
    workspaces: 'M12 2L2 7L12 12L22 7Z M2 17L12 22L22 17 M2 12L12 17L22 12',
    borders: 'M3 3H21V21H3Z M7 7H17V17H7Z',
    mouse: 'M5 3L19 10L13 12L11 18Z',
    animations: 'M13 2L3 14H12L11 22L21 10H12Z',
    bar: 'M3 3H21V21H3Z M3 9H21',
    keybindings: 'M2 6H22V18H2Z M6 10H8 M11 10H13 M16 10H18 M7 14H17',
    rules: 'M3 6L5 8L9 4 M3 14L5 16L9 12 M13 6H21 M13 12H21 M13 18H21',
    search: 'M11 4A7 7 0 1 1 11 18A7 7 0 1 1 11 4Z M21 21L16 16',
    chevron: 'M6 9L12 15L18 9',
    reset: 'M3 12A9 9 0 1 0 6 5.3 M3 3V8H8',
    minimize: 'M5 12H19',
    maximize: 'M5 5H19V19H5Z',
    close: 'M6 6L18 18 M18 6L6 18',
    grip: 'M9 5V6 M15 5V6 M9 11.5V12.5 M15 11.5V12.5 M9 18V19 M15 18V19',
    plus: 'M12 5V19 M5 12H19',
    trash: 'M4 7H20 M9 7V4H15V7 M6 7L7 20H17L18 7',
    edit: 'M4 20H8L19 9L15 5L4 16Z M13 7L17 11',
    lock: 'M6 11H18V21H6Z M8 11V7A4 4 0 0 1 16 7V11',
    warning: 'M12 3L22 20H2Z M12 10V14 M12 17V17.5',
    check: 'M5 12L10 17L19 7',
    clock: 'M12 3A9 9 0 1 1 12 21A9 9 0 1 1 12 3Z M12 7V12L15 14',
    calendar: 'M3 5H21V21H3Z M3 10H21 M8 3V7 M16 3V7',
    ram: 'M3 7H21V17H3Z M7 10V14 M11 10V14 M15 10V14 M19 17V20 M5 17V20',
    cpu: 'M6 6H18V18H6Z M10 10H14V14H10Z M9 2V6 M15 2V6 M9 18V22 M15 18V22 M2 9H6 M2 15H6 M18 9H22 M18 15H22',
    download: 'M12 3V15 M7 10L12 15L17 10 M4 20H20',
    pause: 'M8 5V19 M16 5V19',
    music: 'M9 18V5L20 3V16 M9 18A3 3 0 1 1 3 18A3 3 0 1 1 9 18Z M20 16A3 3 0 1 1 14 16A3 3 0 1 1 20 16Z',
    grid: 'M3 3H10V10H3Z M14 3H21V10H14Z M3 14H10V21H3Z M14 14H21V21H14Z',
    window: 'M3 4H21V20H3Z M3 8H21',
    wifi: 'M2 9A15 15 0 0 1 22 9 M5 13A10 10 0 0 1 19 13 M8.5 16.5A5 5 0 0 1 15.5 16.5 M12 20V20.5',
    volume: 'M4 9H8L13 5V19L8 15H4Z M16 9A4 4 0 0 1 16 15 M19 6A8 8 0 0 1 19 18',
    up: 'M6 15L12 9L18 15',
};

// Icons with no height are sized by width.
const WIDE_ICONS = new Set(['minimize']);

module.exports = { FLAVORS, roles, ICONS, WIDE_ICONS };
