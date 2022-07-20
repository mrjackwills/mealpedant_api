import { customTypeError } from './customError';
import { WWW_DOMAIN } from '../config/env';
import { TEmailInterface } from '../types';
import mjml2html from 'mjml';

export const createEmailTemplate = (data: TEmailInterface): string=> {
	if (!data.title || !data.firstName || !data.lineOne) throw customTypeError('createEmailTemplate: !title || !name || !lineOne');
	if (data.buttonText && !data.buttonLink || !data.buttonText && data.buttonLink) throw customTypeError('createEmailTemplate: buttonText && !buttonLink || !buttonText && buttonLink');
	const full_domain = `https://${WWW_DOMAIN}`;
	const start_section =
`<mjml>
	<mj-head>
		<!-- random comment -->
		<mj-title>${data.title}</mj-title>
		<mj-attributes>
			<mj-all font-family='Open Sans, Tahoma, Arial, sans-serif'></mj-all>
		</mj-attributes>
		<mj-style inline='inline'>
			.link-nostyle { color: inherit; text-decoration: none }
	 	</mj-style>
	</mj-head>
	<mj-body background-color='#929892'>
		<mj-section padding-top='30px'></mj-section>
		<mj-section background-color='#212121' border-radius='10px' text-align='center'>
		<mj-column vertical-align='middle' width='100%'>
			<mj-image width='320px' src='https://static.mealpedant.com/email_header.png'></mj-image>
			<mj-spacer height='15px'></mj-spacer>
				<mj-text line-height='1.2' color='#ffffff' font-weight='500' font-size='20px'>Hi ${data.firstName},</mj-text>
				<mj-text line-height='1.2' color='#ffffff' font-weight='500' font-size='20px'>${data.lineOne}</mj-text>`;
	const lineTwo_section=
`				<mj-text line-height='1.2' color='#ffffff' font-weight='500' font-size='20px'>${data.lineTwo}</mj-text>`;
	const button_section =
`				<mj-button href='${data.buttonLink}' border-radius='10px' background-color='#7ca1b2' font-size='20px'>${data.buttonText}</mj-button>
				<mj-text line-height='1.2' align='center' color='#ffffff' font-size='13px'>
					or copy and paste this address into the browser address bar
				</mj-text>
				<mj-text line-height='1.2' align='center' color='#ffffff' font-size='13px'>
					<a class='link-nostyle'>
						${data.buttonLink}
					</a>
				</mj-text>`;
	const end_section =
`			</mj-column>
			<mj-column vertical-align='middle' width='100%' padding-top='40px' >
				<mj-text line-height='1.2' align='center' color='#ffffff' font-size='12px'> This is an automated email - replies sent to this email address are not read<br></br>
					<a class='link-nostyle' href='${full_domain}'>
						${full_domain}
					</a> © 2015 - 
				</mj-text> 
			</mj-column>
		</mj-section>
		<mj-section padding-bottom='30px'></mj-section>
	</mj-body>
</mjml>`;
	const combined_sections = data.lineTwo && data.buttonLink
		? `${start_section}${lineTwo_section}${button_section}${end_section}`
		: data.lineTwo && !data.buttonLink
			?`${start_section}${lineTwo_section}${end_section}`
			: !data.lineTwo && data.buttonText
				? `${start_section}${button_section}${end_section}`
				: `${start_section}${end_section}`;
	const htmlOutput = mjml2html(combined_sections, { keepComments: false, minifyOptions: { removeEmptyAttributes: true, minifyCSS: true, collapseWhitespace: true } });
	if (htmlOutput.errors.length >0 && htmlOutput.errors[0]?.message) throw customTypeError(htmlOutput.errors[0].message);
	return htmlOutput.html;
};