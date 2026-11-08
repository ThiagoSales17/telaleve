from pathlib import Path
import shutil, subprocess
import argparse
parser=argparse.ArgumentParser(description="Empacotar TelaLeve no Debian 13 amd64")
parser.add_argument('--appdir', type=Path)
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--linuxdeploy', type=Path, required=True)
parser.add_argument('--appimagetool', type=Path, required=True)
parser.add_argument('--runtime', type=Path, required=True)
parser.add_argument('--nice-plugin', type=Path, default=Path('/usr/lib/x86_64-linux-gnu/gstreamer-1.0/libgstnice.so'))
args=parser.parse_args()
base=Path(__file__).resolve().parent.parent
version=__import__('json').loads((base/'package.json').read_text())['version']
app=args.appdir.resolve() if args.appdir else base/'build/TelaLeve.AppDir'
app.parent.mkdir(exist_ok=True)
for folder in ['usr/bin','usr/lib/gstreamer-1.0','usr/lib/webkit2gtk-4.1/injected-bundle','usr/share/applications','usr/share/icons/hicolor/64x64/apps','usr/share/glib-2.0/schemas']:(app/folder).mkdir(parents=True,exist_ok=True)
shutil.copy2(args.binary.resolve(),app/'usr/bin/telaleve')
shutil.copy2(base/'src-tauri/icons/icon.png',app/'usr/share/icons/hicolor/64x64/apps/telaleve.png')
(app/'usr/share/applications/telaleve.desktop').write_text('[Desktop Entry]\nType=Application\nName=TelaLeve\nExec=telaleve\nIcon=telaleve\nTerminal=false\nCategories=Utility;Network;\n')
for name in ['app','coreelements','ximagesrc','videoconvertscale','videobox','vpx','rtp','rtpmanager','webrtc','dtls','srtp','sctp','udp','tcp','videoparsersbad','videotestsrc']:
 shutil.copy2(Path('/usr/lib/x86_64-linux-gnu/gstreamer-1.0')/f'libgst{name}.so',app/'usr/lib/gstreamer-1.0')
shutil.copy2(args.nice_plugin.resolve(),app/'usr/lib/gstreamer-1.0')
for name in ['WebKitWebProcess','WebKitNetworkProcess','WebKitGPUProcess']:shutil.copy2(Path('/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1')/name,app/'usr/bin')
shutil.copy2('/usr/lib/x86_64-linux-gnu/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner',app/'usr/bin')
shutil.copytree('/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/injected-bundle',app/'usr/lib/webkit2gtk-4.1/injected-bundle',dirs_exist_ok=True)
shutil.copytree('/usr/share/glib-2.0/schemas',app/'usr/share/glib-2.0/schemas',dirs_exist_ok=True)
(app/'AppRun').write_text('''#!/bin/sh
set -eu
TELALEVE_APPDIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
export LD_LIBRARY_PATH="$TELALEVE_APPDIR/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export GST_PLUGIN_PATH_1_0="$TELALEVE_APPDIR/usr/lib/gstreamer-1.0${GST_PLUGIN_PATH_1_0:+:$GST_PLUGIN_PATH_1_0}"
export GST_PLUGIN_SCANNER="$TELALEVE_APPDIR/usr/bin/gst-plugin-scanner"
export WEBKIT_EXEC_PATH="$TELALEVE_APPDIR/usr/bin"
export WEBKIT_INJECTED_BUNDLE_PATH="$TELALEVE_APPDIR/usr/lib/webkit2gtk-4.1/injected-bundle"
export GSETTINGS_SCHEMA_DIR="$TELALEVE_APPDIR/usr/share/glib-2.0/schemas"
export XDG_DATA_DIRS="$TELALEVE_APPDIR/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "$TELALEVE_APPDIR/usr/bin/telaleve" "$@"
''');(app/'AppRun').chmod(0o755)
cmd=[str(args.linuxdeploy.resolve()),'--appdir',str(app),'--deploy-deps-only',str(app/'usr'),'--desktop-file',str(app/'usr/share/applications/telaleve.desktop'),'--icon-file',str(app/'usr/share/icons/hicolor/64x64/apps/telaleve.png')]
with open(app.parent/'linuxdeploy.log','w')as log:subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,check=True)

for name in ['gstreamer1.0-plugins-base','gstreamer1.0-plugins-good','gstreamer1.0-plugins-bad','gstreamer1.0-x','gstreamer1.0-nice']:
    copyright=Path('/usr/share/doc')/name/'copyright'
    if copyright.exists():
        dest=app/'usr/share/doc'/name
        dest.mkdir(exist_ok=True,parents=True)
        shutil.copy2(copyright,dest/'copyright')
output=base/'dist'/f'TelaLeve-{version}-x86_64.AppImage'
output.parent.mkdir(exist_ok=True)
env=__import__('os').environ.copy();env['ARCH']='x86_64';env['VERSION']=version
subprocess.run([str(args.appimagetool.resolve()),'--no-appstream','--runtime-file',str(args.runtime.resolve()),'--mksquashfs-opt','-processors','--mksquashfs-opt','4',str(app),str(output)],env=env,check=True)
print(output)
