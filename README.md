<h1 align="center">Token staking</h1>
<br />

<div align="center">
<img src="https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white" />
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

</div>

<br />
<a name="program-id"></a>
<h2 align="center">Program ID</h2>
<br />

- mainnet: `---TBD---`
- devnet: `3F15CLnQjHqMCHLn2g7vuDULiRuJDiEMSZQXMoVVUGtA`

<br />
<a name="audit"></a>
<h2 align="center">Audit</h2>
<br />

This code has been audited ✅

- Forked codebase by Kudelski: [Bonfida Token Vesting Report](/audit/Bonfida_SecurityAssessment_Vesting_Final050521.pdf)
- Modified codebase: `---TBD---`

<br />
<a name="overview"></a>
<h2 align="center">Overview</h2>
<br />

- The codebase is a modified fork of the Bonfida Token Vesting program, into a Staking program.
- The staking contract allows you to deposit X SPL tokens that will get unlocked at a certain block height/slot.
- Allows a pre-defined list of possible time periods for staking: 0 for "unlocked", 3 mth, 6, 9, 12.
- On "unlocked" stakes, there is a 7 day withdrawal period since the user initializes the withdrawal.
