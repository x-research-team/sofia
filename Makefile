.PHONY: update-readme readme-rollback readme-status

update-readme:
	python3 scripts/update_readme.py --trigger=manual

readme-rollback:
	python3 scripts/update_readme.py --rollback

readme-status:
	@cat .readme_sync.log 2>/dev/null || echo "No sync logs found"
