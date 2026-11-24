"""Read-only proposal for an exact pair, one-run maintainer permit.

This validates a positive GitHub Environment review. There is intentionally
no dispatch, secret, registry write, retry, approval mutation or claim store.
"""
import hashlib
import json
import re


PACKAGE_ORDER = ("@farmfe/docs-theme-default", "@farmfe/docs")
REPOSITORY = "farm-fe/farm"


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def positive_id(value):
    return type(value) is int and 0 < value < 10**20


def sha(value, size):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{" + str(size) + r"}", value) is not None


def approval_comment(expected):
    """Canonical exact permit, printed by the credential-free inspection job."""
    fields = {"artifactId", "artifactSha256", "archives", "environmentId", "sourceSHA",
              "sourceTree", "producerRun", "writerRun", "attempt", "tag"}
    require(type(expected) is dict and set(expected) == fields, "Invalid exact Docs approval shape")
    require(all(positive_id(expected[key]) for key in ("artifactId", "environmentId", "producerRun", "writerRun"))
            and expected["writerRun"] != expected["producerRun"]
            and type(expected["attempt"]) is int and expected["attempt"] == 1,
            "Exact original producer and first writer attempt required")
    require(sha(expected["sourceSHA"], 40) and sha(expected["sourceTree"], 40)
            and sha(expected["artifactSha256"], 64), "Invalid original source or artifact digest")
    require(expected["tag"] == "farm-docs-bootstrap", "No stable tag writes")
    archives = expected["archives"]
    require(type(archives) is dict and set(archives) == set(PACKAGE_ORDER)
            and all(sha(archives[name], 64) for name in PACKAGE_ORDER), "Exact original pair required")
    body = {"operation": "farm-docs-one-shot-token-publish-v1", "repo": REPOSITORY, "version": "2.0.0", **expected}
    canonical = json.dumps(body, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode("ascii")
    return "farm-docs-2.0.0 approve sha256:" + hashlib.sha256(canonical).hexdigest()


def verify_owner_review(expected, *, review_history, trusted_reviewer_ids, actor_id, environment_name):
    """Read live review history for this run through an already scoped reader.

    This is a positive permit for this one run. An empty history is never a
    publication retry signal or proof that no preceding run submitted bytes.
    """
    comment = approval_comment(expected)
    require(type(trusted_reviewer_ids) is frozenset and trusted_reviewer_ids
            and all(positive_id(a) for a in trusted_reviewer_ids)
            and positive_id(actor_id)
            and isinstance(environment_name, str) and environment_name,
            "Reviewed environment and maintainer IDs required")
    reviews = review_history(f"/actions/runs/{expected['writerRun']}/approvals")
    require(type(reviews) is list and len(reviews) <= 64, "Invalid or unbounded deployment review history")
    matched = []
    for item in reviews:
        require(type(item) is dict and type(item.get("environments")) is list
                and len(item["environments"]) <= 64, "Invalid deployment review record")
        environments = item["environments"]
        require(all(type(env) is dict and positive_id(env.get("id")) and isinstance(env.get("name"), str)
                    for env in environments), "Invalid deployment environment identity")
        target = [env for env in environments if env["id"] == expected["environmentId"]
                  or env["name"] == environment_name]
        if target:
            require(len(environments) == 1 and len(target) == 1
                    and target[0]["id"] == expected["environmentId"] and target[0]["name"] == environment_name,
                    "Review does not uniquely identify the protected environment")
            matched.append(item)
    require(len(matched) == 1, "Exactly one maintainer decision required")
    decision = matched[0]
    actor = decision.get("user")
    require(type(actor) is dict and positive_id(actor.get("id")) and actor["id"] != actor_id
            and actor["id"] in trusted_reviewer_ids, "Review is not from a separate approved maintainer")
    require(decision.get("state") == "approved" and decision.get("comment") == comment,
            "Approval does not bind this exact first release run and original archives")
    return True
