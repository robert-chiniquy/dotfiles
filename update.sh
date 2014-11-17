# source into your env

[ -e .venv ] || virtualenv -p python2.7 .venv
. .venv/bin/activate
pip install -r requirements.txt

echo "fab dotbox:host=core@<IP>" 
