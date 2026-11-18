ObjC.import('AppKit');
function run(args) {
    if (args.length < 1 || args.length > 2) throw new Error('Expected app bundle path');
    const app = args[0];
    if (args[1] === '--clear') {
        if (!$.NSWorkspace.sharedWorkspace.setIconForFileOptions($(), $(app), 0))
            throw new Error('Cannot clear custom icon');
        return;
    }
    const icon = $.NSImage.alloc.initWithContentsOfFile($(app + '/Contents/Resources/AppIcon.icns'));
    if (!icon || icon.isNil()) throw new Error('Cannot load bundle icon');
    if (!$.NSWorkspace.sharedWorkspace.setIconForFileOptions(icon, $(app), 0)) {
        throw new Error('Cannot apply custom icon to ' + app);
    }
}
